<script lang="ts">
  import { onMount } from 'svelte'
  import { cubicOut } from 'svelte/easing'
  import { slide } from 'svelte/transition'
  import Edit from './Edit.svelte'
  import type { Group } from './types'
  import { invisibleCursorFix } from './helpers'

  export let onCreate: (group: Group) => void
  let editMode = false

  let textarea: HTMLTextAreaElement
  onMount(resize)
  function resize() {
    if (textarea) {
      textarea.style.height = ''
      textarea.style.height = textarea.scrollHeight + 'px'
    }
  }
  function onInput() {
    resize()
  }

  let group: Group = newBlank()
  function newBlank(): Group {
    return {
      id: 0,
      enabled: true,
      title: '',
      description: '',
      nextDate: new Date(),
      repeat: 'never',
    }
  }
  $: editMode = group.title + group.description !== ''
</script>

<form
  class="mb-3 flex w-full flex-col rounded bg-white bg-opacity-10"
  class:p-3.5={editMode}
  on:submit|preventDefault
>
  <input
    class="w-full rounded-t-sm border-none bg-white bg-opacity-0 px-2 py-1 text-sm focus:ring-0"
    class:bg-opacity-10={editMode}
    placeholder={editMode ? 'Title' : 'New reminder...'}
    type="text"
    tabindex="0"
    bind:value={group.title}
    use:invisibleCursorFix
  />
  {#if editMode}
    <textarea
      bind:this={textarea}
      rows="1"
      class="w-full resize-none rounded-b-sm border-none bg-white bg-opacity-0 px-2 py-1 text-xs text-white text-opacity-75 focus:ring-0"
      class:bg-opacity-10={editMode}
      placeholder="Description"
      tabindex={editMode ? 0 : -1}
      type="text"
      bind:value={group.description}
      on:input={onInput}
      transition:slide={{ easing: cubicOut }}
    />
    <Edit
      bind:group
      onCancel={() => {
        editMode = false
        group = newBlank()
      }}
      onSave={() => {
        editMode = false
        onCreate(group)
        group = newBlank()
      }}
    />
  {/if}
</form>