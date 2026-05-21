<script lang="ts">
  type Props = {
    code: string;
    language?: "json" | "text";
    class?: string;
  };

  let { code, language = "text", class: className = "" }: Props = $props();

  function escapeHtml(value: string): string {
    return value
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }

  function highlightJson(value: string): string {
    const escaped = escapeHtml(value);
    return escaped.replace(
      /("(?:\\u[\da-fA-F]{4}|\\[^u]|[^\\"])*"(?=\s*:)|"(?:\\u[\da-fA-F]{4}|\\[^u]|[^\\"])*"|true|false|null|-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/g,
      (token, offset, source) => {
        let cls = "syntax-number";
        if (token.startsWith('"')) {
          const rest = source.slice(offset + token.length);
          cls = /^\s*:/.test(rest) ? "syntax-key" : "syntax-string";
        }
        else if (token === "true" || token === "false") cls = "syntax-boolean";
        else if (token === "null") cls = "syntax-null";
        return `<span class="${cls}">${token}</span>`;
      },
    );
  }

  const rendered = $derived(language === "json" ? highlightJson(code) : escapeHtml(code));
</script>

<pre class={className}><code>{@html rendered}</code></pre>
