<script>
  export let open = false;
  export let dismissible = true;
  export let onClose = () => {
    open = false;
  };
</script>

{#if open}
  <div class="popup-background" on:click={() => {if (dismissible) {onClose()}}}>
    <div class="popup" on:click={(e) => e.stopPropagation()}>
      {#if dismissible}
        <button class="close" on:click={onClose}>x</button>
      {/if}
      <slot></slot>
    </div>
  </div>
{/if}


<style lang="scss">
  .popup-background {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .popup {
    position: relative;
    background-color: white;
    padding: 20px;
    border-radius: 5px;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.1);
  }

  .close {
    position: absolute;
    top: -10px;
    right: -10px;
    background-color: #f0f0f0;
    border-radius: 50%;
    border: none;
    cursor: pointer;
    width: 24px;
    height: 24px;
    display: flex;
    justify-content: center;
    align-items: center;
  }
</style>