<script lang="ts">
  import { DateInput } from 'date-picker-svelte'
  import { slide, fly } from 'svelte/transition'
  import type { Group } from './types'

  export let group: Group
  export let onSave: () => void
  export let onCancel = () => {
    /* noop */
  }
</script>

<!-- <div class="mt-2" transition:slide={{ easing: cubicOut }}> -->
<div in:slide={{ duration: 200 }} out:slide={{ duration: 300 }}>
  <!-- <div transition:fly={{ y: -10, delay: 100 }}> -->
  <div in:fly={{ duration: 300, y: -20 }} out:fly={{ duration: 200, y: -20 }}>
    <div class="mt-2">
      <DateInput
        bind:value={group.nextDate}
        closeOnSelection={true}
        --date-picker-background="#031212"
        --date-picker-foreground="#f7f7f7"
        --date-input-width="100%"
        --date-picker-highlight-border="hsl(183, 98%, 49%)"
        --date-picker-highlight-shadow="hsla(183, 98%, 49%, 50%)"
        --date-picker-selected-color="hsl(183, 100%, 85%)"
        --date-picker-selected-background="hsla(183, 98%, 49%, 20%)"
      />
    </div>
    <div class="flex w-full">
      <button
        type="button"
        class="mt-2 mr-2 w-full cursor-default rounded-sm bg-[#576f70] px-2 py-1 text-sm"
        on:click|stopPropagation={() => onCancel()}>Cancel</button
      >
      <button
        type="submit"
        class="mt-2 w-full cursor-default rounded-sm bg-[#31898c] px-2 py-1 text-sm"
        on:click|stopPropagation={() => onSave()}>Save</button
      >
    </div>
  </div>
</div>
