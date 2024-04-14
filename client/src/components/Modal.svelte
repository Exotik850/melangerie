<script lang="ts">
  import { checkUser, createUser, loginUser } from "$lib/index";
  import { token_store } from "$lib/stores";
  import { redirect } from "@sveltejs/kit";
  let isLoading = false;
  let hasError = false;
  let hasWhiteSpace = false;
  let isLogin = true;
  let username: string = "";
  let password: string = "";
  let timeStamp = new Date();
  async function handleSignup() {
    const userExists = await checkUser(username);
    if (userExists) return false;
    return await createUser(username, password);
  }

  const handleSubmit = async () => {
    hasWhiteSpace = username.includes(" ");
    if (hasWhiteSpace) {
      isLoading = false;
      return;
    }
    isLoading = true; // Sanitize username
    hasError = false;
    const result = isLogin
      ? await loginUser(username, password)
      : await handleSignup();

    isLoading = false;
    hasError = !result;

    if (result) {
      $token_store = result;
    }
  };
  const toggleMode = () => {
    isLogin = !isLogin;
    hasError = false;
    hasWhiteSpace = false;
    username = "";
    password = "";
  };
</script>

<div class="backdrop" id="vanta">
  <div class="modal">
    <div class="add-username">
      <form on:submit|preventDefault={handleSubmit}>
        <input
          type="text"
          placeholder="Enter your username here"
          bind:value={username}
        />
        <!-- Password input -->
        <input
          type="password"
          placeholder="Enter your password here"
          bind:value={password}
        /> <button type="submit">{isLogin ? "Login" : "Sign Up"}</button>
        {#if isLoading}
          <small>Loading...</small>
        {/if}
        {#if hasError && isLogin}
          <small>Invalid username or password. Please try again.</small>
        {/if}
        {#if hasError && !isLogin}
          <small>The username already exists. Please try again.</small>
        {/if}
        {#if hasWhiteSpace}
          <small>Invalid username. Must not contain any spaces.</small>
        {/if}
        {#if !hasError && !isLoading && !hasWhiteSpace && !isLogin}
          <small>Your username will expire after <span>24 hrs.</span></small>
        {/if}
        <button type="button" on:click={toggleMode}>
          {isLogin ? "Create an account" : "Login to an existing account"}
        </button>
      </form>
    </div>
  </div>
</div>

<style lang="scss">
  .backdrop {
    width: 100%;
    height: 100%;
    position: fixed;
    background: rgba(0, 0, 0, 0.8);
    top: 0;
    left: 0;
    display: grid;
    place-items: center;
  }
  .add-username {
    padding: 20px;
    border-radius: 10px;
    margin: auto;

    button {
      background-color: #3ecf8e;
      color: white;
      border: none;
      padding: 10px 20px;
      border-radius: 5px;
      cursor: pointer;
      transition: background-color 0.3s ease;

      &:hover {
        background-color: #2ca97f;
      }
    }

    input {
      border: none;
      padding: 20px 40px;
      border-radius: 50px;
      text-align: center;
      display: block;
      margin: auto;
      margin-bottom: 10px;
      box-shadow: 0px 5px #3ecf8e;
      transition: all ease 0.1s;

      &:focus {
        outline: none;
        box-shadow: 0px 10px #3ecf8e;
        margin-bottom: 20px;
      }
    }

    span {
      color: #3ecf8e;
      font-weight: 600;
    }
  }
</style>
