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
</script>

{#if $activeCollection}
  <div class="flex h-full">
    <!-- Left: endpoint list -->
    <div class="w-56 shrink-0">
      <EndpointList
        endpoints={$activeCollection.endpoints}
        activeId={$activeEndpointId}
        collectionId={$activeCollection.id}
        onSelect={(id) => activeEndpointId.set(id)}
        onAdd={() => addEndpoint($activeCollection!.id)}
        onDelete={(id) => deleteEndpoint($activeCollection!.id, id)}
        onDuplicate={(id) => duplicateEndpoint($activeCollection!.id, id)}
      />
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
          <span class="text-sm text-ink-muted font-krub">Select an endpoint</span>
          <span class="text-xs text-ink-dim">or create one with +</span>
        </div>
      {/if}
    </div>
  </div>
{/if}
