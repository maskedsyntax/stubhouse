<script lang="ts">
  type Props = {
    filename?: string;
    language?: string;
    code: string;
    class?: string;
  };

  let { filename, language = '', code, class: className = '' }: Props = $props();

  let copied = $state(false);

  type TokenRule = {
    regex: RegExp;
    className: string;
  };

  const yamlValueRules: TokenRule[] = [
    { regex: /#.*$/y, className: 'tok-cmt' },
    { regex: /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/y, className: 'tok-str' },
    { regex: /\b(?:true|false|null)\b/y, className: 'tok-lit' },
    { regex: /\b\d+(?:\.\d+)?\b/y, className: 'tok-num' },
    { regex: /[{}[\],|]/y, className: 'tok-punct' }
  ];

  const bashRules: TokenRule[] = [
    { regex: /#.*$/y, className: 'tok-cmt' },
    { regex: /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/y, className: 'tok-str' },
    { regex: /\$[A-Z_][A-Z0-9_]*/y, className: 'tok-var' },
    { regex: /\b[A-Z_][A-Z0-9_]*(?==)/y, className: 'tok-var' },
    { regex: /\b(?:stubhouse|npm|kill|cargo|git|cd)\b/y, className: 'tok-fn' },
    { regex: /--?[a-zA-Z0-9-]+/y, className: 'tok-key' },
    { regex: /\b\d+(?:ms|s)?\b/y, className: 'tok-num' },
    { regex: /[&|;]/y, className: 'tok-punct' }
  ];

  const rhaiRules: TokenRule[] = [
    { regex: /\/\/.*$/y, className: 'tok-cmt' },
    { regex: /"(?:\\.|[^"\\])*"/y, className: 'tok-str' },
    { regex: /\b(?:test|let|if|else|return|true|false|null)\b/y, className: 'tok-key' },
    { regex: /\b(?:response|status|json|time_ms)\b/y, className: 'tok-var' },
    { regex: /\b[A-Za-z_][A-Za-z0-9_]*(?=\()/y, className: 'tok-fn' },
    { regex: /\b\d+(?:\.\d+)?\b/y, className: 'tok-num' },
    { regex: /[{}()[\].!=<>+\-*/]/y, className: 'tok-punct' }
  ];

  function escapeHtml(value: string): string {
    return value
      .replaceAll('&', '&amp;')
      .replaceAll('<', '&lt;')
      .replaceAll('>', '&gt;');
  }

  function span(className: string, value: string): string {
    return `<span class="${className}">${escapeHtml(value)}</span>`;
  }

  function highlightInline(src: string, rules: TokenRule[]): string {
    let out = '';
    let i = 0;

    while (i < src.length) {
      let matched = false;

      for (const rule of rules) {
        rule.regex.lastIndex = i;
        const match = rule.regex.exec(src);
        if (!match || match.index !== i || match[0].length === 0) continue;

        out += span(rule.className, match[0]);
        i += match[0].length;
        matched = true;
        break;
      }

      if (!matched) {
        out += escapeHtml(src[i]);
        i += 1;
      }
    }

    return out;
  }

  function highlightYamlLine(line: string): string {
    const commentOnly = /^(\s*)(#.*)$/.exec(line);
    if (commentOnly) return `${escapeHtml(commentOnly[1])}${span('tok-cmt', commentOnly[2])}`;

    const keyValue = /^(\s*)(-\s*)?([A-Za-z_][\w-]*)(:)(.*)$/.exec(line);
    if (!keyValue) return highlightInline(line, yamlValueRules);

    const [, indent, dash = '', key, colon, value] = keyValue;
    return [
      escapeHtml(indent),
      dash ? span('tok-punct', dash) : '',
      span('tok-key', key),
      span('tok-punct', colon),
      highlightInline(value, yamlValueRules)
    ].join('');
  }

  function highlightBashLine(line: string): string {
    const prompt = /^(\$\s?)(.*)$/.exec(line);
    if (!prompt) return highlightInline(line, bashRules);
    return `${span('tok-prompt', prompt[1])}${highlightInline(prompt[2], bashRules)}`;
  }

  function escapeAndHighlight(src: string): string {
    const lang = language.toLowerCase();
    return src
      .split('\n')
      .map((line) => {
        if (lang === 'yaml' || lang === 'yml') return highlightYamlLine(line);
        if (lang === 'bash' || lang === 'sh' || lang === 'shell') return highlightBashLine(line);
        if (lang === 'rhai') return highlightInline(line, rhaiRules);
        return highlightInline(line, [...bashRules, ...rhaiRules]);
      })
      .join('\n');
  }

  async function copy() {
    try {
      await navigator.clipboard.writeText(code);
      copied = true;
      setTimeout(() => (copied = false), 2000);
    } catch {
      /* ignore */
    }
  }
</script>

<div class="code-block {className}">
  {#if filename || language}
    <div class="code-block__head">
      {#if filename}<span class="code-block__file mono">{filename}</span>{/if}
      <span class="code-block__lang mono">{language}</span>
      <button type="button" class="code-block__copy mono" onclick={copy} aria-label="Copy code">
        {copied ? 'Copied' : 'Copy'}
      </button>
    </div>
  {/if}
  <pre class="code-block__pre mono"><code>{@html escapeAndHighlight(code)}</code></pre>
</div>

<style>
  .code-block {
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    width: 100%;
    max-width: 100%;
    min-width: 0;
    border-radius: var(--radius-md);
    overflow: hidden;
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.18);
  }

  .code-block__head {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
  }

  .code-block__file {
    flex: 1;
    font-size: var(--text-mono);
    color: var(--text-secondary);
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .code-block__lang {
    font-size: 12px;
    color: var(--text-tertiary);
    text-transform: lowercase;
  }

  .code-block__copy {
    margin-left: auto;
    border: 0;
    background: transparent;
    color: var(--text-tertiary);
    font-size: 12px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    transition: color 150ms ease;
  }

  .code-block__copy:hover,
  .code-block__copy:focus-visible {
    color: var(--text-primary);
    background: var(--shimmer);
  }

  .code-block__pre {
    margin: 0;
    padding: 16px;
    max-width: 100%;
    overflow-x: auto;
    font-size: var(--text-mono);
    line-height: 1.55;
    color: var(--syntax-text);
    background: var(--bg-surface);
  }

  @media (max-width: 639px) {
    .code-block {
      border-radius: var(--radius-sm);
      box-shadow: 0 14px 36px rgba(0, 0, 0, 0.2);
    }

    .code-block__head {
      gap: 8px;
      padding: 9px 10px;
    }

    .code-block__file {
      font-size: 11px;
    }

    .code-block__lang,
    .code-block__copy {
      font-size: 11px;
    }

    .code-block__pre {
      padding: 12px;
      font-size: 11px;
      line-height: 1.55;
      max-height: 360px;
      white-space: pre-wrap;
      overflow-wrap: anywhere;
      word-break: break-word;
    }
  }

  :global(html.theme-dark) .code-block {
    --syntax-text: #d6d3cb;
    --syntax-key: #f4eee0;
    --syntax-fn: #c9d7ff;
    --syntax-str: #b7dfc4;
    --syntax-lit: #f0d79b;
    --syntax-num: #f0d79b;
    --syntax-var: #d7c7ff;
    --syntax-punct: #77746d;
    --syntax-comment: #78746d;
    --syntax-prompt: #8f8a80;
  }

  :global(html.theme-light) .code-block {
    --syntax-text: #383734;
    --syntax-key: #0a0a0a;
    --syntax-fn: #284c8f;
    --syntax-str: #28613f;
    --syntax-lit: #7a4f00;
    --syntax-num: #7a4f00;
    --syntax-var: #5a3d8f;
    --syntax-punct: #8a8780;
    --syntax-comment: #85817a;
    --syntax-prompt: #8a8780;
  }

  .code-block :global(.tok-key),
  .code-block :global(.tok-fn),
  .code-block :global(.tok-str),
  .code-block :global(.tok-lit),
  .code-block :global(.tok-num),
  .code-block :global(.tok-var),
  .code-block :global(.tok-punct),
  .code-block :global(.tok-prompt),
  .code-block :global(.tok-cmt) {
    font-variant-ligatures: none;
  }

  .code-block :global(.tok-key) {
    color: var(--syntax-key);
    font-weight: 500;
  }

  .code-block :global(.tok-fn) {
    color: var(--syntax-fn);
    font-weight: 500;
  }

  .code-block :global(.tok-str) {
    color: var(--syntax-str);
    font-weight: 400;
  }

  .code-block :global(.tok-lit),
  .code-block :global(.tok-num) {
    color: var(--syntax-lit);
  }

  .code-block :global(.tok-var) {
    color: var(--syntax-var);
  }

  .code-block :global(.tok-punct),
  .code-block :global(.tok-prompt) {
    color: var(--syntax-punct);
  }

  .code-block :global(.tok-cmt) {
    color: var(--syntax-comment);
    font-style: italic;
  }
</style>
