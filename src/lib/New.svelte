<script lang="ts">
  import { onMount } from 'svelte'
  import { cubicOut } from 'svelte/easing'
  import { slide } from 'svelte/transition'
  import Edit from './Edit.svelte'
  import type { Group } from './types'
  import { checkShortcut, invisibleCursorFix, runCmd } from './helpers'

  export let onCreate: (newGroups: Group[]) => void
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

  async function create(newGroup: Group) {
    const newGroups: Group[] = await runCmd('new_group', { group: newGroup })
    editMode = false
    onCreate(newGroups)
    group = newBlank()
  }

  let group: Group = newBlank()
  function newBlank(): Group {
    return {
      id: '',
      enabled: true,
      title: '',
      description: '',
      cron: '0 0 12,13 */2 * *',
    }
  }

  let titleInput: HTMLInputElement
  export function open() {
    editMode = true
    titleInput.focus()
  }
  function cancel() {
    editMode = false
    group = newBlank()
  }
</script>

<form
  class="mb-3 flex w-full flex-col rounded bg-white bg-opacity-10"
  class:p-3.5={editMode}
  on:keydown={(e) => {
    if (checkShortcut(e, 'Escape') && editMode) {
      cancel()
      e.preventDefault()
      e.stopPropagation()
    }
  }}
  on:submit|preventDefault={() => {
    if (editMode) {
      create(group)
    } else {
      open()
    }
  }}
>
  <input
    bind:this={titleInput}
    class="w-full rounded-t-sm border-none bg-white px-2 py-1 text-sm focus:ring-0"
    class:bg-opacity-0={!editMode}
    class:bg-opacity-10={editMode}
    placeholder={editMode ? 'Title' : 'New reminder...'}
    type="text"
    tabindex="0"
    bind:value={group.title}
    use:invisibleCursorFix
    on:input={() => {
      if (group.title !== '') {
        editMode = true
      }
    }}
  />
  {#if editMode}
    <textarea
      bind:this={textarea}
      rows="1"
      class="w-full resize-none rounded-b-sm border-none bg-white px-2 py-1 text-xs text-white text-opacity-75 focus:ring-0"
      class:bg-opacity-0={!editMode}
      class:bg-opacity-10={editMode}
      placeholder="Description"
      tabindex={editMode ? 0 : -1}
      bind:value={group.description}
      on:input={onInput}
      transition:slide={{ easing: cubicOut }}
    />
    <Edit bind:group onCancel={cancel} />
  {/if}
</form>
