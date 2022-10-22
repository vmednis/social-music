<script>
  import {Router, Link, Route} from "svelte-navigator";
</script>

<Router>
  <header class="bg-zinc-700 text-white">
    <ul>
      <li><Link to="page1">Page 1</Link></li>
      <li><Link to="page2">Page 2</Link></li>
      <li><a href="/login">Login with Spotify</a></li>
    </ul>
  </header>
  <main>
    <Route path="page1">This is page 1</Route>
    <Route path="page2">This is page 2</Route>
  </main>
</Router>

<style>
</style>
