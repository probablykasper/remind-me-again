<script lang="ts">
  import { slide, fly } from 'svelte/transition'
  import { getCronText, invisibleCursorFix } from './helpers'
  import type { Group } from './types'

  export let group: Group

  export let onCancel = () => {
    /* noop */
  }

  let selectedSegments: number[] = []

  async function checkSegment() {
    requestAnimationFrame(() => {
      selectedSegments = []
      if (!document.activeElement?.isSameNode(cronInput)) {
        return
      }
      const segmentMatches = [...group.cron.matchAll(/\S+/g)]
      let i = 0
      for (const segmentMatch of segmentMatches) {
        const segment = segmentMatch.index
        const start = cronInput.selectionStart
        const end = cronInput.selectionEnd
        if (segment === undefined || start === null || end === null) {
          continue
        }
        if (segment >= start - 1 && segment <= end) {
          selectedSegments.push(i)
        }
        i++
      }
    })
  }

  let cronInput: HTMLInputElement
  function selectSegment(segment: number) {
    cronInput.focus()
    const segmentMatches = [...group.cron.matchAll(/\S+/g)]
    const segmentMatch = segmentMatches[segment]
    const segmentIndex = segmentMatch?.index ?? null
    if (segmentIndex !== null) {
      const segmentText = segmentMatch[0]
      cronInput.setSelectionRange(segmentIndex, segmentIndex + segmentText.length)
    }
  }
</script>

<div in:slide={{ duration: 200 }} out:slide={{ duration: 300 }}>
  <div in:fly={{ duration: 300, y: -20 }} out:fly={{ duration: 200, y: -20 }} class="mt-2 text-sm">
    <p class="mb-1 cursor-auto select-auto text-center text-sm">{getCronText(group.cron)}</p>
    <input
      bind:this={cronInput}
      class="mb-2 w-full rounded-t-sm border-none bg-white bg-opacity-10 px-2 py-1 text-center text-sm leading-normal focus:ring-0"
      placeholder="Value"
      type="text"
      tabindex="0"
      bind:value={group.cron}
      on:mousedown={checkSegment}
      on:mousemove={checkSegment}
      on:mouseup={checkSegment}
      on:keydown={checkSegment}
      on:blur={checkSegment}
      on:focus={checkSegment}
      use:invisibleCursorFix
    />
    <div class="mb-2 flex cursor-default justify-center text-xs text-white">
      {#each ['sec', 'min', 'hour', 'day', 'month', 'weekday'] as segment, i}
        {@const selected = selectedSegments.includes(i)}
        <div
          class="px-1 hover:opacity-100"
          class:opacity-60={!selected}
          class:opacity-100={selected}
          on:click={() => selectSegment(i)}
        >
          {segment}
        </div>
      {/each}
    </div>
    <div class="flex w-full">
      <button
        type="button"
        class="mr-2 w-full cursor-default rounded-sm bg-[#576f70] px-2 py-1 text-sm"
        on:click|stopPropagation={() => onCancel()}>Cancel</button
      >
      <button type="submit" class="w-full cursor-default rounded-sm bg-[#31898c] px-2 py-1 text-sm"
        >Save</button
      >
    </div>
  </div>
</div>
