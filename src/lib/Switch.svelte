<script lang="ts">
  export let value: boolean

  let classes = ''
  export { classes as class }
</script>

<div class="form-check {classes}">
  <label class="flex cursor-pointer justify-between">
    <input class="hidden" type="checkbox" bind:checked={value} />
    <slot />
    <div class="flex items-center justify-between">
      <button
        class="flex h-[1.5rem] w-[2.25rem] items-center rounded-full p-[0.125em] outline-none duration-200 ease-in-out focus:ring-2 focus:ring-[#32b9bd]"
        class:bg-[#335152]={!value}
        class:bg-[#31898c]={value}
        on:click={() => {
          value = !value
        }}
      >
        <div
          class="h-[1.125rem] w-[1.125rem] transform rounded-full bg-white shadow-md duration-200 ease-in-out"
          class:translate-x-[0.825rem]={value}
        />
      </button>
    </div>
  </label>
</div>
