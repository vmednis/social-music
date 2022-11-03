<script>
  import {Router, Link, Route} from "svelte-navigator";
  import RoomPreJoin from "/src/pages/RoomPreJoin.svelte";
  import NewRoom from "/src/pages/NewRoom.svelte";

  let greeting = "No greeting :(";
  fetch("/test").then((response) => response.text()).then((body) => greeting = body);
</script>

<Router>
  <div class="h-screen max-h-screen w-screen flex flex-col overflow-hidden">
    <header class="bg-zinc-700 text-white w-screen p-2 grow-0">
      <ul class="flex flex-row">
        <li><Link to="" class="p-2 hover:bg-zinc-500">Home</Link></li>
        <li><Link to="/room/new" class="p-2 hover:bg-zinc-500">New Room</Link></li>
        <li><a href="/login" class="p-2 hover:bg-zinc-500">Login with Spotify</a></li>
      </ul>
    </header>
    <main class="flex-auto min-h-0">
      <Route path="">
        <p>{greeting}</p>
      </Route>
      <Route path="room/new" component={NewRoom}/>
      <Route path="room/:roomId" component={RoomPreJoin}/>
    </main>
  </div>
</Router>

<style>
</style>
