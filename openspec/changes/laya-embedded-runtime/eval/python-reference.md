# Python 参考实现（Rust 复刻依据）


## laya.common.build_head

```python
def build_head(tok, q: Dict, head_max_len: int = 192, option_order: Optional[List[int]] = None):
    """The question half of a sequence: `[CLS] <type> instructions [SEP] [MASK] opt0 [MASK] opt1 ... [SEP]`.

    `build_sequence` appends the state to this and `state_room` measures what is left over for it,
    so neither can disagree with the other about what the head costs.

    Returns `(ids, markers, stats)`: the head token ids, the `[MASK]` position of each option in
    `option_order`, and the stats dict `build_sequence` returns for `return_stats`.
    """
    mask_tok = tok.mask_token
    opts = render_options(q)
    order = option_order if option_order is not None else list(range(len(opts)))
    ins = str(q["ins"]).replace(mask_tok, " ")
    head_ids = _encode_question_text(tok, "%s question: %s" % (q["t"], ins), add_special_tokens=False)
    opt_ids = []
    for i in order:
        # Cap at the tokenizer, not after the fact: `[:48]` still makes the tokenizer process the
        # whole (possibly long) description. truncation=True, max_length=48 keeps the first 48
        # tokens, which is exactly what the previous slice produced.
        opt_tokens = _encode_question_text(
            tok,
            " " + opts[i].replace(mask_tok, " "),
            add_special_tokens=False,
            truncation=True,
            max_length=48,
        )
        opt_ids.append([tok.mask_token_id] + opt_tokens)
    opt_budget = head_max_len - sum(len(o) for o in opt_ids)
    per_option = None
    if opt_budget < 16:
        per = max(4, (head_max_len - 16) // max(1, len(opt_ids)))
        per_option = per
        opt_ids = [o[:per] for o in opt_ids]
        opt_budget = head_max_len - sum(len(o) for o in opt_ids)
    head_ids = head_ids[: max(8, opt_budget)]
    ids = [tok.cls_token_id] + head_ids + [tok.sep_token_id]
    markers = []
    for o in opt_ids:
        markers.append(len(ids))
        ids.extend(o)
    ids.append(tok.sep_token_id)
    # Two options that share a prefix can come out of the cut as the same token span: the marker
    # count still matches the option count, so the guard in `Agent._encode_state` passes and
    # nothing downstream can tell that the question lost the ability to name them apart. Counted
    # on the capped option ids, before assembly: re-slicing the finished sequence cannot close
    # the last option's span -- it runs on into the serialized state, which differs per request,
    # so the last option always looks distinguishable however it collided (#538).
    return ids, markers, {
        "options": len(opt_ids),
        "options_distinct": len({tuple(o) for o in opt_ids}),
        "tokens_per_option": per_option,
    }

```


## laya.common.render_options

```python
def render_options(q: Dict) -> List[str]:
    """Render option texts in label-index order. Noul semantic order is always [false, true]."""
    t, crit = q["t"], q.get("crit")
    if t != "noul" and "labels" in q:
        raise ValueError("labels is only supported for noul questions")
    if t == "choice":
        # only None/"" mean "no description"; 0 and False are legitimate criterion values.
        # `str(k)` unconditionally: a label with no description is rendered as itself, so an int
        # label used to come back as an int from a function annotated `-> List[str]` and then
        # reached `build_sequence`, which calls `.replace` on it and raised an AttributeError
        # naming neither the question nor the label. With a description the same label already
        # went through `"%s: %s" %` and was a str, which is why only the undescribed form broke.
        # `structured._enum_field` stringifies labels the same way; the returned answer still
        # carries the caller's original label, which is unchanged.
        return [str(k) if v is None or v == "" else "%s: %s" % (k, render_criterion(v))
                for k, v in crit.items()]
    if t == "score":
        return ["level %d: %s" % (i, render_criterion(c)) for i, c in enumerate(crit)]
    crit = crit or {}
    false_label, true_label = _resolve_noul_labels(q.get("labels"))
    false_crit, true_crit = crit.get("false"), crit.get("true")
    return [
        false_label + ": "
        + (render_criterion(false_crit) if false_crit not in (None, "") else "no, the statement does not hold"),
        true_label + ": "
        + (render_criterion(true_crit) if true_crit not in (None, "") else "yes, the statement holds"),
    ]

```


## laya.common._encode_question_text

```python
def _encode_question_text(tok, text, **kwargs):
    scope = _QUESTION_TOKEN_CACHE.get()
    cache = scope["tokens"] if scope is not None and scope["thread"] == threading.get_ident() else None
    if cache is None:
        return encode_text(tok, text, **kwargs)["input_ids"]
    # Key the rendered, mask-sanitized text and encoding settings, not a JSON question:
    # option order, structured criteria and custom noul labels must keep their meaning.
    key = (id(tok), text, tuple(kwargs.items()))
    if key not in cache:
        # Keep the tokenizer alive so its identity cannot be reused within this scope.
        cache[key] = (tok, tuple(encode_text(tok, text, **kwargs)["input_ids"]))
    # Sequence assembly must not mutate token lists retained for later states.
    return list(cache[key][1])

```


## laya.common.encode_text

```python
def encode_text(tok, text, **kwargs):
    """Tokenize `text` while holding the lock a shared fast tokenizer needs."""
    with _TOKENIZE_LOCK:
        return tok(text, **kwargs)

```


## laya.common.serialize_state

```python
def serialize_state(state: Union[str, dict, list]) -> str:
    if isinstance(state, str):
        return state
    return json.dumps(state, ensure_ascii=False)

```


## laya.common.confidence_from_probs

```python
def confidence_from_probs(p: np.ndarray, k: int) -> float:
    """Normalized Shannon entropy confidence: 1 - H(p) / log(k).

    How concentrated the whole distribution is. Useful, but not calibrated: it is not what
    temperature scaling fits and not what the reported ECE measures. See `answer_confidence`.

    The formula is reported only for ``k >= 2``: ``log(1)`` and ``log(0)`` are zero, so a
    distribution over fewer than two options has no entropy scale to divide by. Those inputs
    return ``1.0``, the *maximum* score this function can produce, whatever ``p`` holds -- an
    empty or single-option question reads as perfectly concentrated, and a caller comparing the
    reported `confidence` against any threshold sees a pass. That is not the same as the
    abstention gate passing it: `laya.confidence` gates on `answer_confidence` first and reaches
    this number only when that field is absent. The gate does pass a real one-option answer, for
    a different reason -- a softmax over one logit is exactly ``[1.0]``, so both numbers are
    ``1.0``. `laya.structured` builds such an answer from ``{"enum": ["solo"]}``, and the option
    count is bounded only above (`MAX_OPTIONS`).
    """
    if k < 2:
        return 1.0
    p = p[:k]
    ent = -(p * np.log(np.clip(p, 1e-12, 1.0))).sum()
    return float(np.clip(1.0 - ent / math.log(k), 0.0, 1.0))

```


## laya.common.answer_confidence

```python
def answer_confidence(p: np.ndarray, k: int) -> float:
    """Probability mass on the answer being reported: max(p).

    This is the quantity temperature scaling fits, and the quantity every calibration figure in
    this repository is computed on -- both benchmark harnesses take `conf = max(probs)` before
    calling `ece_score`. The README's gating section relies on the property that goes with it:
    of the answers returned at confidence c, about c of them are right. That property is
    conditional, and the condition is not met by default -- it holds only after the temperatures
    have been fitted and validated on held-out data for this checkpoint and this option count.
    The shipped checkpoints are over-confident: `choice:11+` is fitted at 0.1006, a ~10x
    sharpener that would publish a genuine 0.24 top probability as 0.9999. `clamp_temperature`
    refuses it and applies `TEMP_MIN` (0.5) instead, which still sharpens 2x and publishes that
    same 0.24 as 0.4993 -- so a threshold applied to these numbers still selects below model
    accuracy (issue #394). The clamp bounds the damage; it does not calibrate anything.

    `confidence_from_probs` below reports a different quantity on a different scale and carries
    no such guarantee, so the two must not be compared against the same threshold.
    """
    if k < 1:
        return 1.0
    return float(np.clip(np.max(p[:k]), 0.0, 1.0))

```


## laya.common.unpermute_probs

```python
def unpermute_probs(p: np.ndarray, option_order: Optional[List[int]]) -> np.ndarray:
    """Put a slot-ordered probability row back into the caller's option order.

    `build_sequence` puts option `option_order[s]` in slot `s`, so a model row comes back
    indexed by slot. Everything downstream indexes by option -- `zip(keys, p)` for a choice,
    `arange(k) * p` for a score level, `p[1]` for noul-true -- so the row has to be inverted
    first or the probabilities end up attached to the wrong options, which is silent.

    A missing or mismatched order returns `p` untouched, so the canonical path is unaffected.
    """
    if option_order is None or len(option_order) != len(p):
        return p
    canonical = np.empty_like(p)
    canonical[np.asarray(option_order, dtype=int)] = p
    return canonical

```


## laya.common.resolve_lang_temperatures

```python
def resolve_lang_temperatures(raw: Optional[Dict[str, Any]],
                              base_temperature: Sequence[float]) -> Dict[str, Dict[str, Any]]:
    """Parse the `lang_temperatures` option into `{language: {temperature, temperature_by_options}}`.

    One implementation, because `Agent` and `ONNXAgent` both accept this option and both promise
    the same confidences for it. Reading it with `cfg.get(...)` and `len(...)` before checking the
    shape of either raised `AttributeError` and `TypeError` for exactly the inputs the
    `ValueError` below is written for, after the whole checkpoint had loaded:

        {"de": {"temperature": 2}}       -> TypeError: object of type 'int' has no len()
        {"de": {"temperature": None}}    -> TypeError: object of type 'NoneType' has no len()
        {"de": None}                     -> AttributeError: 'NoneType' object has no attribute 'get'

    A `null` entry or a `null` temperature both mean "inherit the checkpoint's own", which is how
    the `laya-ts` port reads the same option (`agent.ts:322-327`).

    Every value still goes through `clamp_temperature`, and a value the clamp had to replace is named
    in a `RuntimeWarning` for its language: `{"de": {"temperature": [True, 1.5, 2.0]}}` answers
    `[1.0, 1.5, 2.0]` and `[99, 0.01, 1]` answers `[5.0, 0.5, 1.0]`, which is a different confidence
    scale from the one the caller wrote. `temperature_rejected` is what distinguishes a replaced entry
    from an accepted one.
    """
    resolved: Dict[str, Dict[str, Any]] = {}
    for lang, cfg in (raw or {}).items():
        if not isinstance(lang, str):
            raise ValueError("Language override keys must be strings, got %r" % (lang,))
        norm = lang.split("-")[0].lower()
        if cfg is None:
            cfg = {}
        if not isinstance(cfg, dict):
            raise ValueError("Language override %r must be a mapping, got %s"
                             % (lang, type(cfg).__name__))
        t_raw = cfg.get("temperature")
        inherited = t_raw is None
        if inherited:
            t_raw = base_temperature
        if not isinstance(t_raw, (list, tuple)) or len(t_raw) != 3:
            raise ValueError("Language override %r temperature must be a list of 3 floats, got %r"
                             % (lang, t_raw))
        tbo_raw = cfg.get("temperature_by_options") or {}
        if not isinstance(tbo_raw, dict):
            raise ValueError("Language override %r temperature_by_options must be a mapping of "
                             "bucket -> float, got %s" % (lang, type(tbo_raw).__name__))
        temperature = [clamp_temperature(t) for t in t_raw]
        by_options = {k: clamp_temperature(v) for k, v in tbo_raw.items()}
        # An override that omits `temperature` inherits the checkpoint's own list, and the loader
        # that got here has already reported the entries it replaced in it. Repeating that line once
        # per language would bury the overrides a reader is actually holding.
        entries = [] if inherited else [
            ("temperature[%d]" % i, raw, applied)
            for i, (raw, applied) in enumerate(zip(t_raw, temperature))
        ]
        entries += [("%s" % k, v, by_options[k]) for k, v in tbo_raw.items()]
        rejected = ["%s=%r -> %g" % (name, raw, applied)
                    for name, raw, applied in entries if temperature_rejected(raw, applied)]
        if rejected:
            warnings.warn(
                "laya: language override %r ships invalid temperatures or values outside "
                "[%g, %g]; using %s. Treat confidence from the affected entries as uncalibrated."
                % (lang, TEMP_MIN, TEMP_MAX, ", ".join(rejected)),
                RuntimeWarning, stacklevel=2)
        resolved[norm] = {
            "temperature": temperature,
            "temperature_by_options": by_options,
        }
    return resolved

```


## laya.common.temp_bucket

```python
def temp_bucket(qtype: int, k: int) -> str:
    size = "2" if k <= 2 else "3-5" if k <= 5 else "6-10" if k <= 10 else "11+"
    return "%s:%s" % (QTYPE_NAMES[int(qtype)], size)

```


## laya.common.clamp_temperature

```python
def clamp_temperature(t, lo: float = TEMP_MIN, hi: float = TEMP_MAX) -> float:
    """A usable temperature: `t` confined to [lo, hi], falling back to 1.0 if it is not a number.

    A bool is not a number either: `True`/`False` used to float to 1.0/0.0 here and read as
    fitted/sharpening temperatures, the same class of quiet acceptance `check_min_confidence`
    already refuses. A custom `lo`/`hi` still bounds either way.
    """
    if isinstance(t, bool):
        return 1.0
    try:
        t = float(t)
    except (TypeError, ValueError):
        return 1.0
    if t != t or t in (float("inf"), float("-inf")):    # NaN / inf
        return 1.0
    return min(hi, max(lo, t))

```


## laya.onnx_agent.collate_items

```python
def collate_items(batch, pad_id: int):
    items = [it for group in batch for it in group]
    if not items:
        return None
    n, L = len(items), max(len(it["ids"]) for it in items)
    kmax = max(len(it["markers"]) for it in items)
    ids = torch.full((n, L), pad_id, dtype=torch.long)
    att = torch.zeros((n, L), dtype=torch.long)
    mpos = torch.zeros((n, kmax), dtype=torch.long)
    mmask = torch.zeros((n, kmax), dtype=torch.bool)
    has_target = any("target" in it for it in items)
    target = torch.zeros((n, kmax), dtype=torch.float32) if has_target else None

    for i, it in enumerate(items):
        ids[i, : len(it["ids"])] = torch.tensor(it["ids"])
        att[i, : len(it["ids"])] = 1
        k = len(it["markers"])
        mpos[i, :k] = torch.tensor(it["markers"])
        mmask[i, :k] = True
        if has_target and "target" in it:
            if len(it["target"]) > k:
                # Otherwise this lands as "The expanded size of the tensor (k) must match the
                # existing size (kmax)" from inside the assignment, which says nothing about the
                # actual mistake: a target with more entries than the item has options. The limit
                # is this item's own marker count, not the batch-wide kmax: in a mixed-width
                # batch a longer sibling row must not legitimise extra entries (#311).
                raise ValueError(
                    "collate_items: item %d has %d target entries but only %d marker positions; "
                    "a target needs one entry per option" % (i, len(it["target"]), k))
            target[i, : len(it["target"])] = torch.tensor(it["target"], dtype=torch.float32)

    res = {
        "input_ids": ids,
        "attention_mask": att,
        "marker_pos": mpos,
        "marker_mask": mmask,
        "qtype": torch.tensor([it["qtype"] for it in items]),
        "label": torch.tensor([it.get("label", -1) for it in items]),
        "meta": [{k: it[k] for k in it if k not in ("ids", "markers", "target", "layout")} for it in items],
    }
    if target is not None:
        res["target"] = target
    if any("layout" in it for it in items):
        # All or none: a batch that mixes layouts would run half its rows through the wrong masks
        if not all("layout" in it for it in items):
            raise ValueError("collate_items: some items carry a parallel layout and some do not")
        pos = torch.zeros((n, L), dtype=torch.long)
        opt = torch.zeros((n, L), dtype=torch.long)
        for i, it in enumerate(items):
            pos[i, : len(it["ids"])] = torch.tensor(it["layout"]["position_ids"])
            opt[i, : len(it["ids"])] = torch.tensor(it["layout"]["option_ids"])
        res["position_ids"], res["option_ids"] = pos, opt
    return res

```


## laya.onnx_agent.apply_confidence_gate

```python
def apply_confidence_gate(results: List[Dict[str, Any]], min_confidence: Optional[float] = None) -> None:
    """Report the confidence gate's state, on the answers a gate was actually applied to.

    A gate is a policy, and a policy whose application cannot be observed is not one. With
    `min_confidence` set, this writes ``abstention`` -- one of :data:`GATE_STATES` -- onto every
    answer, plus ``abstention_threshold``, so a caller can answer three questions it otherwise
    cannot:

    * what fraction of decisions abstained, rather than inferring it from whether
      ``low_confidence`` happened to be set;
    * how many answers the gate could not decide on, which a boolean cannot express at all;
    * what threshold produced these results -- ``flag_low_confidence`` consumes the threshold and
      drops it, so without this a batch run with per-class thresholds cannot be re-split.

    ``GATE_UNEVALUATED`` is the case a boolean cannot express: the gate ran and the answer carried
    no usable confidence, so the gate could not decide. Reporting that as a pass is the same lie
    as reporting it as a flag.

    **With `min_confidence` unset, this writes nothing.** No ``abstention``, no
    ``abstention_threshold``, no flag. That is the whole contract: an ungated call returns exactly
    the payload it returned before, and the presence of the field -- not a fourth value read out
    of it -- is what tells a caller the gate ran. Call it unconditionally, once per call, in place
    of an `if min_confidence is not None:` guard: that guard is what leaves a path reporting
    nothing at all, which is the state this function exists to distinguish.

    The flag itself stays :func:`flag_low_confidence`'s -- this delegates rather than
    re-implementing the rule, so the boolean and the reported state cannot drift apart.

    A `min_confidence` of exactly ``0.0`` *was* set, so states are reported. Nothing can fall below
    it, so :func:`flag_low_confidence` marks no answer at it and clears whatever a stricter threshold
    left on a reused payload. Every answer carrying a usable confidence therefore reads ``passed``,
    and the threshold echo is what distinguishes that from a real pass at a real threshold.
    """
    if min_confidence is None:
        return
    is_map = isinstance(min_confidence, dict)
    flag_low_confidence(results, min_confidence)
    for res in results:
        answers = res.get("answers") if isinstance(res, dict) else None
        if not isinstance(answers, dict):
            continue
        for a in answers.values():
            if not isinstance(a, dict):
                continue
            if a.get("low_confidence"):
                a["abstention"] = GATE_ABSTAINED
            elif _gate_confidence(a) is None:
                a["abstention"] = GATE_UNEVALUATED
            else:
                a["abstention"] = GATE_PASSED
            # With a per-bucket map, echo the threshold this answer's bucket was actually gated at.
            a["abstention_threshold"] = float(
                resolve_min_confidence(a, min_confidence) if is_map else min_confidence)

```


## QTYPES

```python
 {'choice': 0, 'score': 1, 'noul': 2} 
```


## TEMP bounds: 0.5 5.0
