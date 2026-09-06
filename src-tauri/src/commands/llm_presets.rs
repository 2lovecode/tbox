//! CC Switch Claude preset snapshot + TBox-owned providers.
//! Upstream: farion1231/cc-switch main claudeProviderPresets.ts (2026-09-06).

use serde::Serialize;
use super::llm::{LlmProtocol, LlmPresetView};

#[derive(Debug, Clone, Serialize)]
pub struct LlmPreset {
    pub id: &'static str,
    pub label: &'static str,
    pub default_base_url: &'static str,
    pub default_model: &'static str,
    pub default_protocol: LlmProtocol,
    pub requires_oauth: bool,
}

/// Icon id + theme color aligned with CC Switch `icon` / `iconColor`.
pub fn icon_for(id: &str) -> (&'static str, &'static str) {
    match id {
        "local" => ("local", "#4361EE"),
        "ollama" => ("ollama", "#000000"),
        "custom" => ("custom", "#64748B"),
        "claude-official" | "anthropic" => ("anthropic", "#D4915D"),
        "kimi" | "kimi-for-coding" => ("kimi", "#6366F1"),
        "packycode" => ("packycode", "#3B82F6"),
        "zetaapi" => ("zetaapi", "#6366F1"),
        "apinebula" => ("apinebula", "#0EA5E9"),
        "aicodemirror" => ("aicodemirror", "#000000"),
        "patewayai" => ("pateway", "#6366F1"),
        "fennoai" => ("fenno", "#6366F1"),
        "runapi" => ("runapi", "#6366F1"),
        "shengsuanyun" => ("shengsuanyun", "#F59E0B"),
        "aigocode" => ("aigocode", "#5B7FFF"),
        "qiniu" => ("qiniu", "#00AAE7"),
        "aicoding" => ("aicoding", "#000000"),
        "subrouter" => ("subrouter", "#6366F1"),
        "apikey-fun" => ("apikeyfun", "#22C55E"),
        "claudeapi" | "9527code" => ("claudeapi", "#D4915D"),
        "code0" => ("code0", "#111827"),
        "teamorouter" => ("teamorouter", "#6366F1"),
        "ppio" => ("ppio", "#2874FF"),
        "claudecn" => ("claudecn", "#D4915D"),
        "agent-plan" | "coding-plan" => ("huoshan", "#3370FF"),
        "byteplus" => ("byteplus", "#3370FF"),
        "doubaoseed" => ("doubao", "#3370FF"),
        "siliconflow" => ("siliconflow", "#6E29F6"),
        "siliconflow-en" => ("siliconflow", "#000000"),
        "a6api" => ("a6api", "#6366F1"),
        "atlascloud" => ("atlascloud", "#0EA5E9"),
        "compshare" | "compshare-coding-plan" => ("ucloud", "#000000"),
        "ccsub" => ("ccsub", "#6366F1"),
        "sssaicode" => ("sssaicode", "#000000"),
        "micu" => ("micu", "#000000"),
        "rightcode" => ("rc", "#E96B2C"),
        "etok-ai" => ("etok", "#000000"),
        "cubence" => ("cubence", "#000000"),
        "crazyrouter" => ("crazyrouter", "#000000"),
        "dmxapi" => ("dmxapi", "#6366F1"),
        "sudocode-chat" => ("sudocode", "#6366F1"),
        "sudocode-us" => ("sudocode-us", "#6366F1"),
        "xycai" => ("xycai", "#6366F1"),
        "amux" => ("amux", "#6366F1"),
        "gemini-native" => ("gemini", "#4285F4"),
        "deepseek" => ("deepseek", "#1E88E5"),
        "opencode-go" => ("opencode", "#211E1E"),
        "tencent-token-plan" | "tencent-token-plan-intl"
        | "tencent-token-plan-enterprise-pro" | "tencent-token-plan-enterprise-pro-intl"
        | "tencent-token-plan-enterprise-lite" | "tencent-token-plan-enterprise-lite-intl"
            => ("tencent", "#0052D9"),
        "zhipu-glm" | "zhipu-glm-en" => ("zhipu", "#0F62FE"),
        "baidu-qianfan-coding-plan" | "baidu-qianfan-token-plan" => ("baidu", "#2932E1"),
        "bailian" | "bailian-for-coding" => ("bailian", "#624AFF"),
        "qwencloud" | "qwencloud-for-coding" | "qwencloud-token-plan" => ("qwen", "#6336E7"),
        "stepfun" | "stepfun-en" => ("stepfun", "#16D6D2"),
        "modelscope" => ("modelscope", "#624AFF"),
        "kat-coder" => ("catcoder", "#F97316"),
        "longcat" => ("longcat", "#29E154"),
        "minimax" | "minimax-en" | "minimax-openai" | "minimax-cn" => ("minimax", "#FF6B6B"),
        "bailing" => ("bailing", "#6366F1"),
        "aihubmix" => ("aihubmix", "#006FFB"),
        "cherryin" => ("cherryin", "#EC4899"),
        "relaxycode" => ("relaxcode", "#6366F1"),
        "e-flowcode" => ("eflowcode", "#000000"),
        "openrouter" => ("openrouter", "#6566F1"),
        "therouter" => ("therouter", "#6366F1"),
        "novita-ai" => ("novita", "#6366F1"),
        "github-copilot" => ("github", "#24292F"),
        "codex" => ("openai", "#00A67E"),
        "xai-grok" => ("xai", "#000000"),
        "nvidia" => ("nvidia", "#76B900"),
        "pipellm" => ("pipellm", "#6366F1"),
        "xiaomi-mimo" | "xiaomi-mimo-token-plan-china" => ("xiaomi", "#FF6900"),
        "aws-bedrock-aksk" | "aws-bedrock-api-key" => ("aws", "#FF9900"),
        "jiekou-ai" => ("jiekou", "#6366F1"),
        _ => ("", "#6366F1"),
    }
}

const TBOX_OWN: &[LlmPreset] = &[
    LlmPreset { id: "local", label: "本地（内置引擎）", default_base_url: "", default_model: "", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: false },
    LlmPreset { id: "ollama", label: "Ollama", default_base_url: "http://127.0.0.1:11434", default_model: "llama3.2", default_protocol: LlmProtocol::OllamaNative, requires_oauth: false },
    LlmPreset { id: "custom", label: "自定义端点", default_base_url: "", default_model: "", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: false },
    // MiniMax official endpoints. Anthropic-compatible route is the coding
    // plan endpoint; /v1 is the OpenAI-compatible chat completions API.
    LlmPreset { id: "minimax", label: "MiniMax（Anthropic 兼容）", default_base_url: "https://api.minimax.io/anthropic", default_model: "MiniMax-M2", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "minimax-openai", label: "MiniMax（OpenAI 兼容）", default_base_url: "https://api.minimax.io/v1", default_model: "MiniMax-M2", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: false },
    LlmPreset { id: "minimax-cn", label: "MiniMax 国内（OpenAI 兼容）", default_base_url: "https://api.minimaxi.com/v1", default_model: "MiniMax-M2", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: false },
];

const CC_SWITCH: &[LlmPreset] = &[
    LlmPreset { id: "claude-official", label: "Claude Official", default_base_url: "https://api.moonshot.cn/anthropic", default_model: "kimi-k2.7-code", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "kimi", label: "Kimi", default_base_url: "https://api.moonshot.cn/anthropic", default_model: "kimi-k2.7-code", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "kimi-for-coding", label: "Kimi For Coding", default_base_url: "https://api.kimi.com/coding/", default_model: "kimi-for-coding", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "packycode", label: "PackyCode", default_base_url: "https://www.packyapi.ai", default_model: "anthropic/claude-sonnet-5", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "zetaapi", label: "ZetaAPI", default_base_url: "https://api.zetaapi.ai", default_model: "anthropic/claude-sonnet-5", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "apinebula", label: "APINebula", default_base_url: "https://apinebula.ai", default_model: "anthropic/claude-sonnet-5", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "aicodemirror", label: "AICodeMirror", default_base_url: "https://api.aicodemirror.ai/api/claudecode", default_model: "anthropic/claude-sonnet-5", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "patewayai", label: "PatewayAI", default_base_url: "https://api.pateway.ai", default_model: "anthropic/claude-sonnet-5", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "fennoai", label: "FennoAI", default_base_url: "https://api.fenno.ai", default_model: "anthropic/claude-sonnet-5", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "runapi", label: "RunAPI", default_base_url: "https://runapi.host", default_model: "anthropic/claude-sonnet-5", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "shengsuanyun", label: "Shengsuanyun", default_base_url: "https://router.shengsuanyun.com/api", default_model: "anthropic/claude-sonnet-5", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "aigocode", label: "AIGoCode", default_base_url: "https://api.aigocode.app", default_model: "deepseek/deepseek-v4-flash-0731", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "qiniu", label: "Qiniu", default_base_url: "https://api.qnaigc.com", default_model: "deepseek/deepseek-v4-flash-0731", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "aicoding", label: "AICoding", default_base_url: "https://api.aicoding.inc", default_model: "deepseek/deepseek-v4-flash-0731", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "subrouter", label: "SubRouter", default_base_url: "https://subrouter.ai", default_model: "deepseek/deepseek-v4-flash-0731", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "apikey-fun", label: "APIKEY.FUN", default_base_url: "https://api.apikey.fun", default_model: "deepseek/deepseek-v4-flash-0731", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "claudeapi", label: "ClaudeAPI", default_base_url: "https://gw.apito.ai", default_model: "deepseek/deepseek-v4-flash-0731", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "code0", label: "Code0", default_base_url: "https://code0.ai", default_model: "deepseek/deepseek-v4-flash-0731", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "teamorouter", label: "TeamoRouter", default_base_url: "https://api.teamorouter.com", default_model: "deepseek/deepseek-v4-flash-0731", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "ppio", label: "PPIO", default_base_url: "https://api.ppio.com/anthropic", default_model: "deepseek/deepseek-v4-flash-0731", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "claudecn", label: "ClaudeCN", default_base_url: "https://claudecn.top", default_model: "ark-code-latest", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "agent-plan", label: "火山 Agent Plan", default_base_url: "https://ark.cn-beijing.volces.com/api/plan", default_model: "ark-code-latest", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "coding-plan", label: "火山 Coding Plan", default_base_url: "https://ark.cn-beijing.volces.com/api/coding", default_model: "ark-code-latest", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "byteplus", label: "BytePlus", default_base_url: "https://ark.ap-southeast.bytepluses.com/api/coding", default_model: "ark-code-latest", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "doubaoseed", label: "DouBaoSeed", default_base_url: "https://ark.cn-beijing.volces.com/api/compatible", default_model: "doubao-seed-2-1-pro-260628", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "siliconflow", label: "SiliconFlow", default_base_url: "https://api.siliconflow.cn", default_model: "Pro/MiniMaxAI/MiniMax-M2.5", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "siliconflow-en", label: "SiliconFlow en", default_base_url: "https://api.siliconflow.com", default_model: "MiniMaxAI/MiniMax-M3", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "a6api", label: "A6API", default_base_url: "https://api.a6api.com", default_model: "zai-org/glm-5.1", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "atlascloud", label: "AtlasCloud", default_base_url: "https://api.atlascloud.ai", default_model: "zai-org/glm-5.1", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "compshare", label: "Compshare", default_base_url: "https://api.modelverse.cn", default_model: "gemini-3.6-flash", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "compshare-coding-plan", label: "Compshare Coding Plan", default_base_url: "https://cp.compshare.cn", default_model: "gemini-3.6-flash", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "ccsub", label: "CCSub", default_base_url: "https://www.ccsub.net", default_model: "gemini-3.6-flash", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "sssaicode", label: "SSSAiCode", default_base_url: "https://node-hk.sssaicodeapi.com/api", default_model: "gemini-3.6-flash", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "micu", label: "Micu", default_base_url: "https://www.micuapi.ai", default_model: "gemini-3.6-flash", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "rightcode", label: "RightCode", default_base_url: "https://www.rightapi.ai/claude", default_model: "gemini-3.6-flash", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "etok-ai", label: "ETok.ai", default_base_url: "https://api.etok.ai", default_model: "gemini-3.6-flash", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "cubence", label: "Cubence", default_base_url: "https://api.cubence.com", default_model: "gemini-3.6-flash", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "crazyrouter", label: "CrazyRouter", default_base_url: "https://cn.crazyrouter.com", default_model: "gemini-3.6-flash", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "dmxapi", label: "DMXAPI", default_base_url: "https://www.dmxapi.cn", default_model: "gemini-3.6-flash", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "sudocode-chat", label: "SudoCode.chat", default_base_url: "https://api.sudocode.chat", default_model: "gemini-3.6-flash", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "sudocode-us", label: "SudoCode.us", default_base_url: "https://sudocode.us", default_model: "gemini-3.6-flash", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "xycai", label: "XycAi", default_base_url: "https://apicdn.xycai.us", default_model: "gemini-3.6-flash", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "amux", label: "Amux", default_base_url: "https://api.amux.ai", default_model: "gemini-3.6-flash", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "gemini-native", label: "Gemini Native", default_base_url: "https://generativelanguage.googleapis.com", default_model: "gemini-3.6-flash", default_protocol: LlmProtocol::GeminiNative, requires_oauth: false },
    LlmPreset { id: "deepseek", label: "DeepSeek", default_base_url: "https://api.deepseek.com/anthropic", default_model: "deepseek-v4-pro", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "opencode-go", label: "OpenCode Go", default_base_url: "https://opencode.ai/zen/go", default_model: "deepseek-v4-flash", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "tencent-token-plan", label: "Tencent Token Plan", default_base_url: "https://api.lkeap.cloud.tencent.com/plan/anthropic", default_model: "tc-code-latest", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "tencent-token-plan-intl", label: "Tencent Token Plan (Intl)", default_base_url: "https://tokenhub-intl.tencentcloudmaas.com/plan/anthropic", default_model: "auto", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "tencent-token-plan-enterprise-pro", label: "Tencent Token Plan Enterprise Pro", default_base_url: "https://tokenhub.tencentmaas.com/plan/anthropic", default_model: "auto", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "tencent-token-plan-enterprise-pro-intl", label: "Tencent Token Plan Enterprise Pro (Intl)", default_base_url: "https://tokenhub-intl.tencentmaas.com/plan/anthropic", default_model: "auto", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "tencent-token-plan-enterprise-lite", label: "Tencent Token Plan Enterprise Lite", default_base_url: "https://tokenhub.tencentmaas.com/plan/anthropic", default_model: "auto", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "tencent-token-plan-enterprise-lite-intl", label: "Tencent Token Plan Enterprise Lite (Intl)", default_base_url: "https://tokenhub-intl.tencentmaas.com/plan/anthropic", default_model: "auto", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "zhipu-glm", label: "Zhipu GLM", default_base_url: "https://open.bigmodel.cn/api/anthropic", default_model: "glm-5.1", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "zhipu-glm-en", label: "Zhipu GLM en", default_base_url: "https://api.z.ai/api/anthropic", default_model: "glm-5.1", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "baidu-qianfan-coding-plan", label: "Baidu Qianfan Coding Plan", default_base_url: "https://qianfan.baidubce.com/anthropic/coding", default_model: "qianfan-code-latest", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "baidu-qianfan-token-plan", label: "Baidu Qianfan Token Plan", default_base_url: "https://qianfan.baidubce.com/anthropic", default_model: "ernie-x1", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "bailian", label: "Bailian", default_base_url: "https://dashscope.aliyuncs.com/apps/anthropic", default_model: "step-3.5-flash-2603", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "bailian-for-coding", label: "Bailian For Coding", default_base_url: "https://coding.dashscope.aliyuncs.com/apps/anthropic", default_model: "step-3.5-flash-2603", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "qwencloud", label: "QwenCloud", default_base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1", default_model: "qwen3-coder-plus", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: false },
    LlmPreset { id: "qwencloud-for-coding", label: "QwenCloud For Coding", default_base_url: "https://coding.dashscope.aliyuncs.com/apps/anthropic", default_model: "qwen3-coder-plus", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "qwencloud-token-plan", label: "QwenCloud Token Plan", default_base_url: "https://dashscope.aliyuncs.com/apps/anthropic", default_model: "qwen3-coder-plus", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "stepfun", label: "StepFun", default_base_url: "https://api.stepfun.com/step_plan", default_model: "step-3.5-flash-2603", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "stepfun-en", label: "StepFun en", default_base_url: "https://api.stepfun.ai/step_plan", default_model: "step-3.5-flash-2603", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: true },
    LlmPreset { id: "modelscope", label: "ModelScope", default_base_url: "https://api-inference.modelscope.cn", default_model: "ZhipuAI/GLM-5.2", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: true },
    LlmPreset { id: "kat-coder", label: "KAT-Coder", default_base_url: "https://vanchin.streamlake.ai/api/gateway/v1/endpoints/${ENDPOINT_ID}/claude-code-proxy", default_model: "KAT-Coder-Pro V1", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: true },
    LlmPreset { id: "longcat", label: "Longcat", default_base_url: "https://api.longcat.chat/anthropic", default_model: "LongCat-2.0", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: true },
    LlmPreset { id: "minimax", label: "MiniMax", default_base_url: "https://api.minimaxi.com/anthropic", default_model: "MiniMax-M2.7", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: true },
    LlmPreset { id: "minimax-en", label: "MiniMax en", default_base_url: "https://api.minimax.io/anthropic", default_model: "MiniMax-M2.7", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: true },
    LlmPreset { id: "bailing", label: "BaiLing", default_base_url: "https://api.tbox.cn/api/anthropic", default_model: "Ling-2.5-1T", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: true },
    LlmPreset { id: "aihubmix", label: "AiHubMix", default_base_url: "https://aihubmix.com", default_model: "anthropic/claude-sonnet-5", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: true },
    LlmPreset { id: "cherryin", label: "CherryIN", default_base_url: "https://open.cherryin.net", default_model: "anthropic/claude-sonnet-5", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: true },
    LlmPreset { id: "relaxycode", label: "RelaxyCode", default_base_url: "https://www.relaxycode.com", default_model: "anthropic/claude-sonnet-5", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: true },
    LlmPreset { id: "e-flowcode", label: "E-FlowCode", default_base_url: "https://e-flowcode.cc", default_model: "anthropic/claude-sonnet-5", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: true },
    LlmPreset { id: "openrouter", label: "OpenRouter", default_base_url: "https://openrouter.ai/api", default_model: "anthropic/claude-sonnet-5", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: true },
    LlmPreset { id: "therouter", label: "TheRouter", default_base_url: "https://api.therouter.ai", default_model: "anthropic/claude-sonnet-5", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: true },
    LlmPreset { id: "novita-ai", label: "Novita AI", default_base_url: "https://api.novita.ai/anthropic", default_model: "zai-org/glm-5.1", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: true },
    LlmPreset { id: "github-copilot", label: "GitHub Copilot", default_base_url: "https://api.githubcopilot.com", default_model: "claude-sonnet-5", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: true },
    LlmPreset { id: "codex", label: "Codex", default_base_url: "https://chatgpt.com/backend-api/codex", default_model: "gpt-5.6-sol", default_protocol: LlmProtocol::OpenaiResponses, requires_oauth: true },
    LlmPreset { id: "xai-grok", label: "xAI (Grok)", default_base_url: "https://api.x.ai/v1", default_model: "grok-4.5", default_protocol: LlmProtocol::OpenaiResponses, requires_oauth: true },
    LlmPreset { id: "nvidia", label: "Nvidia", default_base_url: "https://integrate.api.nvidia.com", default_model: "moonshotai/kimi-k2.5", default_protocol: LlmProtocol::OpenaiChat, requires_oauth: false },
    LlmPreset { id: "pipellm", label: "PIPELLM", default_base_url: "https://cc-api.pipellm.ai", default_model: "claude-sonnet-5", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "xiaomi-mimo", label: "Xiaomi MiMo", default_base_url: "https://api.xiaomimimo.com/anthropic", default_model: "mimo-v2.5-pro", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "xiaomi-mimo-token-plan-china", label: "Xiaomi MiMo Token Plan (China)", default_base_url: "https://token-plan-cn.xiaomimimo.com/anthropic", default_model: "mimo-v2.5-pro", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "aws-bedrock-aksk", label: "AWS Bedrock (AKSK)", default_base_url: "https://bedrock-runtime.${AWS_REGION}.amazonaws.com", default_model: "global.anthropic.claude-sonnet-5", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "aws-bedrock-api-key", label: "AWS Bedrock (API Key)", default_base_url: "https://bedrock-runtime.${AWS_REGION}.amazonaws.com", default_model: "global.anthropic.claude-sonnet-5", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
    LlmPreset { id: "jiekou-ai", label: "JieKou AI", default_base_url: "https://api.jiekou.ai/anthropic", default_model: "claude-fable-5", default_protocol: LlmProtocol::AnthropicMessages, requires_oauth: false },
];

pub fn all_presets() -> Vec<LlmPresetView> {
    let mut out = Vec::with_capacity(TBOX_OWN.len() + CC_SWITCH.len());
    for p in TBOX_OWN.iter().chain(CC_SWITCH.iter()) {
        out.push(LlmPresetView::from_preset(p));
    }
    out
}

pub fn find_preset(id: &str) -> Option<&'static LlmPreset> {
    TBOX_OWN.iter().chain(CC_SWITCH.iter()).find(|p| p.id == id)
}

pub fn default_protocol_for_provider(id: &str) -> LlmProtocol {
    find_preset(id)
        .map(|p| p.default_protocol)
        .unwrap_or_else(|| infer_legacy_protocol(id))
}

fn infer_legacy_protocol(id: &str) -> LlmProtocol {
    match id {
        "local" => LlmProtocol::OpenaiChat,
        "ollama" => LlmProtocol::OllamaNative,
        "openai" | "deepseek" | "custom" => LlmProtocol::OpenaiChat,
        "anthropic" => LlmProtocol::AnthropicMessages,
        _ => LlmProtocol::AnthropicMessages,
    }
}

