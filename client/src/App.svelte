<script>
  import {Router, Link, Route} from "svelte-navigator";
</script>

<Router>
  <header class="bg-zinc-700 text-white w-screen p-2">
    <ul class="flex flex-row">
      <li><Link to="page1" class="p-2 hover:bg-zinc-500">Page 1</Link></li>
      <li><Link to="page2" class="p-2 hover:bg-zinc-500">Page 2</Link></li>
      <li><a href="/login" class="p-2 hover:bg-zinc-500">Login with Spotify</a></li>
    </ul>
  </header>
  <main>
    <Route path="page1">This is page 1</Route>
    <Route path="page2">This is page 2</Route>
  </main>
</Router>

<style>
</style>
