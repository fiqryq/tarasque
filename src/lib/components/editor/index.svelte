<script lang="ts">
  import EndpointList from './EndpointList.svelte';
  import EndpointDetail from './EndpointDetail.svelte';
  import {
    activeCollection,
    activeEndpoint,
    activeEndpointId,
    activeCollectionId,
    addEndpoint,
    deleteEndpoint,
    duplicateEndpoint,
    updateEndpoint,
  } from '$lib/stores/collections';

  let listWidth = $state(224);
  let isResizing = $state(false);
  const LIST_MIN = 160;
  const LIST_MAX = 400;

  function startResize(e: MouseEvent) {
    e.preventDefault();
    isResizing = true;
    const startX = e.clientX;
    const startWidth = listWidth;

    function onMouseMove(e: MouseEvent) {
      listWidth = Math.max(LIST_MIN, Math.min(LIST_MAX, startWidth + (e.clientX - startX)));
    }

    function onMouseUp() {
      isResizing = false;
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    }

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }
</script>

{#if $activeCollection}
  <div class="flex h-full {isResizing ? 'select-none cursor-col-resize' : ''}">
    <!-- Left: endpoint list -->
    <div class="shrink-0 relative" style="width: {listWidth}px;">
      <EndpointList
        endpoints={$activeCollection.endpoints}
        activeId={$activeEndpointId}
        collectionId={$activeCollection.id}
        onSelect={(id) => activeEndpointId.set(id)}
        onAdd={() => addEndpoint($activeCollection!.id)}
        onDelete={(id) => deleteEndpoint($activeCollection!.id, id)}
        onDuplicate={(id) => duplicateEndpoint($activeCollection!.id, id)}
      />
      <!-- Resize handle -->
      <div
        class="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-accent/30 transition-colors z-10
               {isResizing ? 'bg-accent/40' : ''}"
        onmousedown={startResize}
        role="separator"
        aria-orientation="vertical"
        tabindex="-1"
      ></div>
    </div>

    <!-- Right: endpoint detail -->
    <div class="flex-1 min-w-0">
      {#if $activeEndpoint && $activeCollectionId}
        <EndpointDetail
          endpoint={$activeEndpoint}
          collectionId={$activeCollectionId}
          onUpdate={(ep) => updateEndpoint($activeCollectionId!, ep)}
        />
      {:else}
        <div class="flex flex-col items-center justify-center h-full gap-1.5">
          <span class="text-sm text-ink-secondary font-sf">No endpoint selected</span>
          <span class="text-xs text-ink-tertiary">Select one from the list or press + to create</span>
        </div>
      {/if}
    </div>
  </div>
{/if}
