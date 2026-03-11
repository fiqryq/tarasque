<script lang="ts">
  import { AlertTriangle } from 'lucide-svelte';

  interface Props {
    title: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
    destructive?: boolean;
    onConfirm: () => void;
    onCancel: () => void;
  }

  let {
    title,
    message,
    confirmLabel = 'Delete',
    cancelLabel = 'Cancel',
    destructive = true,
    onConfirm,
    onCancel,
  }: Props = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onCancel();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed inset-0 z-[60] flex items-center justify-center bg-black/40 backdrop-blur-sm"
  onclick={onCancel}
  role="dialog"
  aria-modal="true"
  aria-labelledby="confirm-title"
  aria-describedby="confirm-message"
>
  <div
    class="glass-card w-[380px] shadow-hig-dark overflow-hidden"
    onclick={(e) => e.stopPropagation()}
  >
    <div class="p-6">
      <div class="flex items-start gap-4">
        {#if destructive}
          <div class="w-10 h-10 rounded-full bg-err/12 flex items-center justify-center shrink-0">
            <AlertTriangle size={20} class="text-err" />
          </div>
        {/if}
        <div class="flex-1 min-w-0">
          <h3 id="confirm-title" class="text-title text-ink">{title}</h3>
          <p id="confirm-message" class="text-body text-ink-secondary mt-2 leading-relaxed">{message}</p>
        </div>
      </div>
    </div>

    <div class="hig-separator"></div>
    <div class="flex justify-end gap-3 px-6 py-4">
      <button onclick={onCancel} class="btn-capsule btn-capsule-secondary">
        {cancelLabel}
      </button>
      <button
        onclick={onConfirm}
        class="btn-capsule {destructive ? 'bg-err text-white hover:bg-err/90' : 'btn-capsule-primary'}"
      >
        {confirmLabel}
      </button>
    </div>
  </div>
</div>
