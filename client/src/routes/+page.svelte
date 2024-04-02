<script lang="ts">
  import { get } from "svelte/store";
  import Modal from "../components/Modal.svelte";
  import { uname_store } from "../stores";
  import { onMount } from "svelte";
  import { checkUser } from "$lib";

  let uname = get(uname_store);

  // onMount(() => {
  //   if (uname) {
  //     if (!checkUser(uname)) {
  //       localStorage.removeItem("OCusername");
  //       location.reload();
  //     }
  //   }
  // });
</script>

<main> 
  {#if uname}
    You're logged in as {uname}
  {:else}
    <Modal />
  {/if}
</main>