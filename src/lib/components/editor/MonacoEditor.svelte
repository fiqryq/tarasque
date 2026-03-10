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
    
    try {
      const content = editor.getValue();
      const parsed = JSON.parse(content);
      const formatted = JSON.stringify(parsed, null, 2);
      editor.setValue(formatted);
      jsonError = null;
    } catch (e) {
      jsonError = (e as Error).message;
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
  
  {#if jsonError}
    <div class="px-3 py-2 bg-rose-500/10 border-t border-rose-500/30 flex items-start gap-2">
      <span class="text-rose-400 text-[11px] font-mono shrink-0">✕</span>
      <span class="text-rose-400 text-[11px] font-mono truncate">{jsonError}</span>
      <button
        onclick={formatJson}
        class="ml-auto px-2 py-1 text-[10px] font-mono bg-rose-500/20 text-rose-400 rounded hover:bg-rose-500/30 transition-colors shrink-0"
      >
        Format
      </button>
    </div>
  {/if}
</div>