<script lang="ts">
  import { toast } from "svelte-french-toast";
  import { uname } from "$lib/stores";
  import { host } from "$lib";

  async function sendReport() {
    let issue = prompt("Enter the issue you are facing:");
    if (!$uname || !issue) {
      toast.error("Cannot send an empty report!");
      return;
    }
    let res = await fetch(host + `/report`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name: $uname, issue }),
    });
    if (res.status === 200) {
      toast.success("Issue reported successfully!");
    } else {
      toast.error("Failed to report issue! Please try again later.");
    }
  }

</script>

<div class="nav">
  <h1>Chat App</h1>
  <div>
    <button on:click={sendReport}>Report Issue</button>
    <button on:click={() => (window.location.href = "/profile")}>Time </button>
    <button on:click={() => (window.location.href = "/")}>Logout</button>
  </div>
</div>

<style lang="scss">
  .nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background-color: #767575;
    color: white;
    height: 5vh;
    div {
      display: flex;
      gap: 1rem;
    }
    button {
      padding: 0.5rem 1rem;
      border: none;
      background-color: #999898;
      color: white;
      cursor: pointer;
    }
  }
</style>

