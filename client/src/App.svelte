<script>
  import {Router, Link, Route} from "svelte-navigator";
  import RoomPreJoin from "/src/pages/RoomPreJoin.svelte";
  import RoomNew from "/src/pages/RoomNew.svelte";

  let is_logged_in = false;
  if(document.cookie.match(/^(.*;)?\s*userid\s*=\s*[^;]+(.*)?$/)) {
    is_logged_in = true;
  }

  let greeting = "No greeting :(";
  if(is_logged_in) {
    fetch("/test").then((response) => response.text()).then((body) => greeting = body);
  }
</script>

<Router>
{#if is_logged_in}
  <div class="h-screen max-h-screen w-screen flex flex-col overflow-hidden">
    <header class="bg-zinc-700 text-white w-screen p-2 grow-0">
      <ul class="flex flex-row">
        <li><Link to="" class="p-2 hover:bg-zinc-500">Home</Link></li>
        <li><Link to="/room/new" class="p-2 hover:bg-zinc-500">New Room</Link></li>
      </ul>
    </header>
    <main class="flex-auto min-h-0">
      <Route path="">
        <p>{greeting}</p>
      </Route>
      <Route path="room/new" component={RoomNew}/>
      <Route path="room/:roomId" component={RoomPreJoin}/>
    </main>
  </div>
{:else}
  <Route path="">
    <a href="/login">Login with Spotify</a>
  </Route>
{/if}

</Router>

<style>
</style>
