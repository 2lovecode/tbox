/**
 * Markdown 渲染：markdown-it + highlight.js 语法高亮 + DOMPurify 消毒。
 * 供聊天助手消息与思考过程使用；用户消息不做 Markdown（spec 约定）。
 */
import MarkdownIt from 'markdown-it';
import type { MarkdownIt as MarkdownItType, Token } from 'markdown-it';
import hljs from 'highlight.js/lib/common';
import DOMPurify from 'dompurify';
import { prepareJsonFenceBody, prepareMarkdownSource } from '@/utils/jsonDisplay';

// 常用别名：模型输出常用但不在 hljs 注册名里的写法
const LANG_ALIASES: Record<string, string> = {
  toml: 'ini',
  sh: 'bash',
  shell: 'bash',
  zsh: 'bash',
  yml: 'yaml',
  md: 'markdown',
  py: 'python',
  ts: 'typescript',
  js: 'javascript',
  golang: 'go',
  'c++': 'cpp',
};

function resolveLang(raw: string): string {
  const lang = (raw || '').trim().toLowerCase();
  if (!lang) return '';
  const aliased = LANG_ALIASES[lang] ?? lang;
  return hljs.getLanguage(aliased) ? aliased : '';
}

const md: MarkdownItType = new MarkdownIt({
  linkify: true,
  breaks: true,
  highlight(code: string, rawLang: string): string {
    const lang = resolveLang(rawLang);
    const label = escapeHtml((rawLang || '').trim() || 'text');
    const body =
      lang === 'json' || (rawLang || '').trim().toLowerCase() === 'json'
        ? prepareJsonFenceBody(code)
        : code;
    // 复制按钮：无内联事件（消毒会剥离），由全局委托处理器接管
    // （见 setupMarkdownCodeCopy），复制内容取兄弟 <code> 的 textContent。
    const copyBtn =
      '<button type="button" class="md-code-copy" title="复制代码" aria-label="复制代码"><i class="fas fa-copy" aria-hidden="true"></i></button>';
    if (lang) {
      try {
        return `<pre class="md-code-block">${copyBtn}<span class="md-code-lang">${label}</span><code>${hljs.highlight(body, { language: lang, ignoreIllegals: true }).value}</code></pre>`;
      } catch {
        /* fall through to plain */
      }
    }
    return `<pre class="md-code-block">${copyBtn}<span class="md-code-lang">${label}</span><code>${escapeHtml(body)}</code></pre>`;
  },
});

type RenderFn = NonNullable<MarkdownItType['renderer']['rules']['link_open']>;

// 外链安全：新窗口打开 + noopener（消毒后统一加属性）
const defaultLinkRender: RenderFn =
  md.renderer.rules.link_open ??
  ((tokens, idx, options, _env, self) => self.renderToken(tokens, idx, options));

md.renderer.rules.link_open = (tokens: Token[], idx, options, env, self) => {
  const token = tokens[idx];
  token.attrSet('target', '_blank');
  token.attrSet('rel', 'noopener noreferrer');
  return defaultLinkRender(tokens, idx, options, env, self);
};

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

/** 渲染 Markdown 为消毒后的 HTML（防 XSS：脚本与事件处理器被移除）。 */
export function renderMarkdown(source: string): string {
  const prepared = prepareMarkdownSource(source ?? '');
  const raw = md.render(prepared);
  return DOMPurify.sanitize(raw, {
    FORBID_TAGS: ['style', 'iframe', 'object', 'embed', 'form'],
    FORBID_ATTR: ['onerror', 'onload', 'onclick', 'style'],
    ADD_ATTR: ['title', 'aria-label'],
  });
}

// ---------------------------------------------------------------------------
// 代码块复制按钮（事件委托）
//
// v-html 渲染的代码块无法直接绑 Vue 事件，且内联 onclick 会被 DOMPurify
// 剥掉，所以在 document 上做一次全局委托：点击 .md-code-copy 时复制其
// 所属 <pre> 内 <code> 的 textContent，并在按钮上短暂显示成功态。
// ---------------------------------------------------------------------------

let delegationInstalled = false;

async function writeClipboard(text: string): Promise<boolean> {
  try {
    if (navigator?.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    /* fall through to execCommand */
  }
  try {
    const ta = document.createElement('textarea');
    ta.value = text;
    ta.setAttribute('readonly', '');
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.appendChild(ta);
    ta.select();
    const ok = document.execCommand('copy');
    document.body.removeChild(ta);
    return ok;
  } catch {
    return false;
  }
}

function onCodeCopyClick(event: Event): void {
  const target = event.target as HTMLElement | null;
  const btn = target?.closest?.('.md-code-copy') as HTMLButtonElement | null;
  if (!btn) return;
  const pre = btn.closest('pre.md-code-block');
  const code = pre?.querySelector('code');
  if (!code) return;
  void writeClipboard(code.textContent ?? '').then((ok) => {
    if (!ok) return;
    const icon = btn.querySelector('i');
    if (icon) {
      icon.className = 'fas fa-check';
      btn.classList.add('copied');
      window.setTimeout(() => {
        icon.className = 'fas fa-copy';
        btn.classList.remove('copied');
      }, 1500);
    }
  });
}

/** 安装全局代码块复制委托（幂等，main.ts 启动时调用一次）。 */
export function setupMarkdownCodeCopy(): void {
  if (delegationInstalled || typeof document === 'undefined') return;
  delegationInstalled = true;
  document.addEventListener('click', onCodeCopyClick);
}
