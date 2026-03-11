<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type * as Monaco from 'monaco-editor';
  import { settings } from '$lib/stores/settings';

  interface Props {
    value: string;
    language?: string;
    onChange: (value: string) => void;
  }

  let { value, language = 'json', onChange }: Props = $props();

  let container: HTMLDivElement;
  let editor = $state<Monaco.editor.IStandaloneCodeEditor | null>(null);
  let monaco: typeof Monaco | null = null;
  let jsonError = $state<string | null>(null);

  function validateJson(content: string) {
    if (language !== 'json') {
      jsonError = null;
      return;
    }
    
    try {
      JSON.parse(content);
      jsonError = null;
    } catch (e) {
      jsonError = (e as Error).message;
    }
  }

  function formatJson() {
    if (!editor || language !== 'json') return;

    const content = editor.getValue().trim();
    if (!content) return;

    // Try parsing as-is first
    try {
      const parsed = JSON.parse(content);
      const formatted = JSON.stringify(parsed, null, 2);
      editor.setValue(formatted);
      jsonError = null;
      return;
    } catch { /* fall through to fixups */ }

    // Attempt common fixes: trailing commas, single quotes, unquoted keys
    let fixed = content
      .replace(/,\s*([\]}])/g, '$1')           // trailing commas
      .replace(/'/g, '"')                        // single quotes → double quotes
      .replace(/(\w+)\s*:/g, '"$1":');           // unquoted keys

    try {
      const parsed = JSON.parse(fixed);
      const formatted = JSON.stringify(parsed, null, 2);
      editor.setValue(formatted);
      jsonError = null;
    } catch (e) {
      jsonError = (e as Error).message;
    }
  }

  function prettify() {
    if (!editor) return;
    if (language === 'json') {
      formatJson();
    } else {
      // For non-JSON, use Monaco's built-in formatter
      editor.getAction('editor.action.formatDocument')?.run();
    }
  }

  onMount(async () => {
    monaco = await import('monaco-editor');

    editor = monaco.editor.create(container, {
      value,
      language,
      theme: 'vs-dark',
      automaticLayout: true,
      minimap: { enabled: false },
      fontSize: $settings.codeFontSize,
      lineNumbers: 'on',
      scrollBeyondLastLine: false,
      wordWrap: 'on',
      tabSize: 2,
      formatOnPaste: true,
      formatOnType: true,
    });

    editor.onDidChangeModelContent(() => {
      const content = editor!.getValue();
      onChange(content);
      validateJson(content);
    });

    validateJson(value);
  });

  $effect(() => {
    if (editor) {
      editor.updateOptions({ fontSize: $settings.codeFontSize });
    }
  });

  $effect(() => {
    // editor is $state so this effect re-runs when editor becomes non-null (after onMount).
    // value is from $props() so it's tracked. Both dependencies are properly captured.
    if (editor && editor.getValue() !== value) {
      editor.setValue(value);
      validateJson(value);
    }
  });

  onDestroy(() => {
    editor?.dispose();
  });
</script>

<div class="h-full w-full flex flex-col">
  <div bind:this={container} class="flex-1 min-h-0"></div>

  <div class="px-3 py-1.5 border-t border-line flex items-center gap-2 bg-surface">
    {#if jsonError}
      <span class="text-rose-400 text-[10px] font-mono shrink-0">✕</span>
      <span class="text-rose-400 text-[10px] font-mono truncate flex-1">{jsonError}</span>
    {:else}
      <span class="flex-1"></span>
    {/if}
    <button
      onclick={prettify}
      class="px-2 py-0.5 text-[10px] font-mono rounded transition-colors shrink-0
        {jsonError
          ? 'bg-rose-500/20 text-rose-400 hover:bg-rose-500/30'
          : 'bg-accent/10 text-accent hover:bg-accent/20'}"
    >
      Prettify
    </button>
  </div>
</div>