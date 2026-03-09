<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type * as Monaco from 'monaco-editor';

  interface Props {
    value: string;
    language?: string;
    onChange: (value: string) => void;
  }

  let { value, language = 'json', onChange }: Props = $props();

  let container: HTMLDivElement;
  let editor: Monaco.editor.IStandaloneCodeEditor | null = null;
  let monaco: typeof Monaco | null = null;

  onMount(async () => {
    monaco = await import('monaco-editor');

    editor = monaco.editor.create(container, {
      value,
      language,
      theme: 'vs-dark',
      automaticLayout: true,
      minimap: { enabled: false },
      fontSize: 13,
      lineNumbers: 'on',
      scrollBeyondLastLine: false,
      wordWrap: 'on',
      tabSize: 2,
    });

    editor.onDidChangeModelContent(() => {
      onChange(editor!.getValue());
    });
  });

  // Sync external value changes without moving cursor
  $effect(() => {
    if (editor && editor.getValue() !== value) {
      const model = editor.getModel();
      if (model) {
        model.pushEditOperations(
          [],
          [{ range: model.getFullModelRange(), text: value }],
          () => null
        );
      }
    }
  });

  onDestroy(() => {
    editor?.dispose();
  });
</script>

<div bind:this={container} class="h-full w-full"></div>
