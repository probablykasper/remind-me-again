<script lang="ts">
  import { onMount, tick } from 'svelte'
  import type { Group } from './types'
  import Switch from './Switch.svelte'
  import ClickOutside from 'svelte-click-outside'
  import { checkShortcut, invisibleCursorFix } from './helpers'
  import Edit from './Edit.svelte'

  export let group: Group
  export let onDelete: () => void
  let editMode = false

  let card: HTMLElement
  let titleInput: HTMLInputElement
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

  function onClickOutside() {
    cancel()
  }

  let originalGroup: string | null = null
  function startEdit() {
    if (!editMode) {
      editMode = true
      originalGroup = JSON.stringify(group)
    }
  }
  function cancel() {
    editMode = false
    if (originalGroup) {
      group = JSON.parse(originalGroup)
      if (group.nextDate !== null) {
        group.nextDate = new Date(group.nextDate)
      }
    }
  }
  function save() {
    editMode = false
    card.focus()
  }

  async function keydown(e: KeyboardEvent) {
    if (checkShortcut(e, 'Escape')) {
      e.preventDefault()
      cancel()
      await tick()
      card.focus()
    }
  }
  function keydownSelf(e: KeyboardEvent) {
    if (checkShortcut(e, 'Enter')) {
      startEdit()
      titleInput.focus()
      e.preventDefault()
    } else if (checkShortcut(e, 'Backspace')) {
      onDelete()
      e.preventDefault()
    }
  }
</script>

<ClickOutside on:clickoutside={onClickOutside}>
  <form
    on:submit|preventDefault={save}
    bind:this={card}
    class="group mb-3 flex w-full cursor-default items-center rounded-lg p-3.5 text-left shadow-xl outline-none transition-colors duration-150 ease-out focus:bg-[#133134] active:bg-[#133134]"
    class:bg-[#0E2426]={group.enabled}
    class:bg-[#133134]={editMode}
    on:keydown={keydown}
    tabindex={editMode ? null : 0}
    on:keydown|self={keydownSelf}
    on:mousedown={(e) => {
      if (editMode) {
        e.stopPropagation()
      }
    }}
  >
    <div class="mr-3.5 rounded-md p-2">
      {#if group.enabled}
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="#3edce5"
          ><path
            d="M15 21c0 1.598-1.392 3-2.971 3s-3.029-1.402-3.029-3h6zm.137-17.055c-.644-.374-1.042-1.07-1.041-1.82v-.003c.001-1.172-.938-2.122-2.096-2.122s-2.097.95-2.097 2.122v.003c.001.751-.396 1.446-1.041 1.82-4.668 2.709-1.985 11.715-6.862 13.306v1.749h20v-1.749c-4.877-1.591-2.193-10.598-6.863-13.306zm-3.137-2.945c.552 0 1 .449 1 1 0 .552-.448 1-1 1s-1-.448-1-1c0-.551.448-1 1-1zm-6.451 16c1.189-1.667 1.605-3.891 1.964-5.815.447-2.39.869-4.648 2.354-5.509 1.38-.801 2.956-.76 4.267 0 1.485.861 1.907 3.119 2.354 5.509.359 1.924.775 4.148 1.964 5.815h-12.903z"
          /></svg
        >
      {:else}
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="currentColor"
          ><path
            d="M22 17.251v1.749h-13.008l2.205-2h7.254c-1.015-1.422-1.465-3.248-1.798-4.949l1.735-1.574c.561 2.98 1.016 5.928 3.612 6.774zm-9.971 6.749c1.578 0 2.971-1.402 2.971-3h-6c0 1.598 1.449 3 3.029 3zm10.971-19.75l-20.654 18.734-1.346-1.479 2.762-2.505h-1.762v-1.749c4.877-1.591 2.194-10.597 6.863-13.306.645-.374 1.041-1.069 1.04-1.82v-.003c0-1.172.939-2.122 2.097-2.122s2.097.95 2.097 2.122v.003c-.001.75.396 1.447 1.04 1.82 1.076.624 1.759 1.585 2.236 2.711l4.285-3.886 1.342 1.48zm-12-2.25c0 .552.448 1 1 1s1-.448 1-1c0-.551-.448-1-1-1s-1 .449-1 1zm-5.032 15l9.812-8.898c-.353-1.083-.842-1.961-1.646-2.427-1.312-.76-2.888-.801-4.267 0-1.485.862-1.907 3.119-2.353 5.51-.36 1.924-.776 4.148-1.965 5.815h.419z"
          /></svg
        >
      {/if}
    </div>
    <div
      class="mr-auto flex w-full flex-col"
      on:click|preventDefault={() => {
        editMode = true
      }}
    >
      <input
        bind:this={titleInput}
        class="w-full rounded-t-sm border-none bg-white bg-opacity-0 px-2 py-1 text-sm focus:ring-0"
        class:bg-opacity-10={editMode}
        tabindex={editMode ? 0 : -1}
        placeholder="Title"
        type="text"
        bind:value={group.title}
        use:invisibleCursorFix
      />
      <textarea
        bind:this={textarea}
        rows="1"
        class="w-full resize-none rounded-b-sm border-none bg-white bg-opacity-0 px-2 py-1 text-xs text-white text-opacity-75 focus:ring-0"
        class:bg-opacity-10={editMode}
        tabindex={editMode ? 0 : -1}
        placeholder="Description"
        type="text"
        bind:value={group.description}
        on:input={onInput}
      />
      {#if editMode}
        <Edit bind:group onSave={save} onCancel={cancel} />
      {/if}
    </div>
    <div on:click|preventDefault|stopPropagation>
      <Switch class="ml-3.5" bind:value={group.enabled} />
    </div>
  </form>
</ClickOutside>

<style lang="sass">
  // fix tailwind styles
  :global(.date-time-field input)
    background-color: hsla(0, 0%, 100%, 0.1)
    border: none
    font-size: 12px
  :global(.date-time-picker select)
    background-image: none
  :global(.date-time-picker select)
    background-image: none
  :global(.date-time-picker svg)
    box-sizing: content-box
</style>