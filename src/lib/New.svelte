<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { cubicOut } from 'svelte/easing'
  import { slide } from 'svelte/transition'
  import Edit from './Edit.svelte'
  import type { Group } from './types'
  import { checkShortcut, invisibleCursorFix, runCmd } from './helpers'
  import { event } from '@tauri-apps/api'

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
  function newReminder() {
    editMode = true
    titleInput.focus()
  }
  function cancel() {
    editMode = false
    group = newBlank()
  }

  const unlistenFuture = event.listen('tauri://menu', ({ payload }) => {
    if (payload === 'New Reminder') {
      newReminder()
    }
  })
  onDestroy(async () => {
    const unlisten = await unlistenFuture
    unlisten()
  })
</script>

<form
  class="mb-3 flex w-full flex-col rounded bg-white bg-opacity-10"
  class:p-3.5={editMode}
  on:keydown={(e) => {
    if (checkShortcut(e, 'Escape')) {
      cancel()
      e.preventDefault()
    }
  }}
  on:submit|preventDefault={() => {
    if (editMode) {
      create(group)
    } else {
      newReminder()
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
      type="text"
      bind:value={group.description}
      on:input={onInput}
      transition:slide={{ easing: cubicOut }}
    />
    <Edit bind:group onCancel={cancel} />
  {/if}
</form>
