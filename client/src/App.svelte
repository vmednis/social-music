<script>
  import {Router, Link, Route} from "svelte-navigator";
  import RoomPreJoin from "/src/pages/RoomPreJoin.svelte";
  import RoomNew from "/src/pages/RoomNew.svelte";
  import NotFound from "/src/pages/NotFound.svelte";
  import RoomsList from "/src/pages/RoomsList.svelte";
  import LandingPage from "/src/pages/LandingPage.svelte";

  let is_logged_in = false;
  if(document.cookie.match(/^(.*;)?\s*userid\s*=\s*[^;]+(.*)?$/)) {
    is_logged_in = true;
  }
</script>

<Router>
  {#if is_logged_in}
    <div class="h-screen max-h-screen w-screen flex flex-col overflow-hidden">
      <header class="bg-zinc-700 text-white w-screen p-2 grow-0 flex flex-row justify-between">
        <ul class="flex flex-row">
          <li><Link to="" class="p-2 hover:bg-zinc-500">Home</Link></li>
          <li><Link to="/create-room" class="p-2 hover:bg-zinc-500">New Room</Link></li>
        </ul>
        <ul></ul>
        <ul class="flex flex-row">
          <li><a href="/logout" class="p-2 hover:bg-zinc-500">Log out</a></li>
        </ul>
      </header>
      <main class="flex-auto min-h-0">
        <Route path="/" component={RoomsList}/>
        <Route path="/create-room" component={RoomNew}/>
        <Route path="/room/:roomId" component={RoomPreJoin}/>

        <!--Fallback 404-->
        <Route component={NotFound}/>
      </main>
    </div>
  {:else}
    <Route path="/" component={LandingPage}/>

    <!--Fallback 404-->
    <Route component={NotFound}/>
  {/if}
</Router>