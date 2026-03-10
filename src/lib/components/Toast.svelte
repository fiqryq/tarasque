<script lang="ts">
  import { toasts, type Toast } from '$lib/stores/toasts';
  import { X, CheckCircle, AlertCircle, Info } from 'lucide-svelte';

  const ICONS: Record<Toast['type'], typeof CheckCircle> = {
    success: CheckCircle,
    error: AlertCircle,
    info: Info,
  };

  const STYLES: Record<Toast['type'], string> = {
    success: 'border-emerald-500/30 bg-emerald-500/10 text-emerald-400',
    error: 'border-rose-500/30 bg-rose-500/10 text-rose-400',
    info: 'border-blue-500/30 bg-blue-500/10 text-blue-400',
  };
</script>

<div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
  {#each $toasts as toast (toast.id)}
    {@const Icon = ICONS[toast.type]}
    <div
      class="flex items-center gap-3 px-4 py-3 rounded-lg border shadow-lg {STYLES[toast.type]} max-w-sm"
    >
      <Icon size={16} />
      <span class="text-sm text-ink flex-1">{toast.message}</span>
      <button
        onclick={() => toasts.dismiss(toast.id)}
        class="text-ink-dim hover:text-ink"
      >
        <X size={14} />
      </button>
    </div>
  {/each}
</div>