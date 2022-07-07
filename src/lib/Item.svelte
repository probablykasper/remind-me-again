<script lang="ts">
  import { onMount } from 'svelte'
  import type { Group } from './types'
  import { DateInput } from 'date-picker-svelte'
  import { slide } from 'svelte/transition'
  import { cubicOut } from 'svelte/easing'

  export let group: Group
  let editMode = true

  let textarea: HTMLTextAreaElement
  onMount(resize)
  function resize() {
    textarea.style.height = ''
    textarea.style.height = textarea.scrollHeight + 'px'
  }
  function onInput() {
    resize()
  }
</script>

<button
  class="group my-3 flex w-full cursor-default items-center rounded-lg p-3.5 text-left shadow-xl outline-none transition-colors duration-150 ease-out focus:ring-2 focus:ring-[#31898c]"
  class:bg-[#0E2426]={group.enabled}
  on:click={() => {
    if (!editMode) {
      group.enabled = !group.enabled
    }
  }}
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
    class:pointer-events-none={!editMode}
    on:click={(e) => {
      if (editMode) {
        e.preventDefault()
      }
    }}
  >
    <input
      class="w-full rounded-sm border-none bg-white bg-opacity-0 px-2 py-1 text-sm focus:ring-2 focus:ring-[#31898c]"
      class:bg-opacity-10={editMode}
      tabindex={editMode ? 0 : -1}
      type="text"
      bind:value={group.title}
    />
    <div class="transition-all duration-200 ease-out" class:mt-2={editMode} />
    <textarea
      bind:this={textarea}
      rows="1"
      class="w-full resize-none rounded-sm border-none bg-white bg-opacity-0 px-2 py-1 text-xs text-white text-opacity-75 focus:ring-2 focus:ring-[#31898c]"
      class:bg-opacity-10={editMode}
      tabindex={editMode ? 0 : -1}
      type="text"
      bind:value={group.description}
      on:input={onInput}
    />
    {#if editMode}
      <div class="mt-2" transition:slide={{ easing: cubicOut }}>
        <DateInput
          --date-picker-background="#1b1e27"
          --date-picker-foreground="#f7f7f7"
          --date-input-width="100%"
        />
      </div>
    {/if}
  </div>
  <button
    class="ml-3.5 scale-90 rounded-md bg-[#01484c] bg-opacity-50 p-2 text-[#3edce5] opacity-0 outline-none transition-all duration-150 ease-out hover:bg-opacity-70 focus:scale-100 focus:opacity-100 focus:ring-2 focus:ring-[#31898c] group-hover:scale-100 group-hover:opacity-100 group-focus:scale-100 group-focus:opacity-100"
    on:click={(e) => {
      editMode = !editMode
      e.stopPropagation()
      e.preventDefault()
    }}
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="currentColor"
      ><path
        d="M14.078 7.061l2.861 2.862-10.799 10.798-3.584.723.724-3.585 10.798-10.798zm0-2.829l-12.64 12.64-1.438 7.128 7.127-1.438 12.642-12.64-5.691-5.69zm7.105 4.277l2.817-2.82-5.691-5.689-2.816 2.817 5.69 5.692z"
      /></svg
    >
  </button>
</button>

<style lang="sass">
  :global(.date-time-field input)
    background-color: hsla(0, 0, 100, 0.1)
    border: none
    font-size: 12px
</style>
