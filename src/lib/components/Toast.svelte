<script lang="ts">
  import { toasts, type Toast } from '$lib/stores/toasts';
  import { X, CheckCircle, AlertCircle, Info } from 'lucide-svelte';

  const ICONS: Record<Toast['type'], typeof CheckCircle> = {
    success: CheckCircle,
    error: AlertCircle,
    info: Info,
  };

  const STYLES: Record<Toast['type'], string> = {
    success: 'text-run',
    error: 'text-err',
    info: 'text-accent',
  };
</script>

<div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2" aria-live="polite" aria-label="Notifications">
  {#each $toasts as toast (toast.id)}
    {@const Icon = ICONS[toast.type]}
    <div class="glass-card flex items-center gap-3 px-4 py-3 shadow-hig-dark max-w-sm">
      <Icon size={16} class={STYLES[toast.type]} />
      <span class="text-body text-ink flex-1">{toast.message}</span>
      <button
        onclick={() => toasts.dismiss(toast.id)}
        class="w-5 h-5 flex items-center justify-center rounded-full text-ink-tertiary
               hover:text-ink hover:bg-[var(--color-hover)] transition-colors"
        aria-label="Dismiss notification"
      >
        <X size={12} />
      </button>
    </div>
  {/each}
</div>
