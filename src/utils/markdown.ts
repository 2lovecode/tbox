/**
 * Markdown 渲染：markdown-it + highlight.js 语法高亮 + DOMPurify 消毒。
 * 供聊天助手消息与思考过程使用；用户消息不做 Markdown（spec 约定）。
 */
import MarkdownIt from 'markdown-it';
import type { MarkdownIt as MarkdownItType, Token } from 'markdown-it';
import hljs from 'highlight.js/lib/common';
import DOMPurify from 'dompurify';

const md: MarkdownItType = new MarkdownIt({
  linkify: true,
  breaks: true,
  highlight(code: string, lang: string): string {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return `<pre class="md-code-block"><span class="md-code-lang">${escapeHtml(
          lang,
        )}</span><code>${hljs.highlight(code, { language: lang, ignoreIllegals: true }).value}</code></pre>`;
      } catch {
        /* fall through to plain */
      }
    }
    return `<pre class="md-code-block"><code>${escapeHtml(code)}</code></pre>`;
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
  const raw = md.render(source ?? '');
  return DOMPurify.sanitize(raw, {
    FORBID_TAGS: ['style', 'iframe', 'object', 'embed', 'form'],
    FORBID_ATTR: ['onerror', 'onload', 'onclick', 'style'],
  });
}
