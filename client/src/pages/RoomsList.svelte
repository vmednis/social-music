<script>
import { Link } from "svelte-navigator";

let rooms = [];
let ready = false;
fetch(`/api/v1/rooms/`, {
  method: "GET",
  headers: {
    "Accept": "application/json",
  }
}).then((response) => {
  if(response.ok) {
    response.text().then((body) => {
      rooms = JSON.parse(body);
      ready = true;
    });
  } else {
    console.log("Failed to retrieve rooms list.");
  }
});

</script>

<div class="w-full h-full grow bg-slate-50 flex justify-center">
  {#if ready}
    <div class="w-full max-w-2xl">
      <h1 class="text-xl">Open Rooms:</h1>
      <table class="table-auto w-full">
        <thead>
          <tr>
            <th scope="col" class="font-bold text-left">Actions</th>
            <th scope="col" class="font-bold text-left">Id</th>
            <th scope="col" class="font-bold text-left">Name</th>
            <th scope="col" class="font-bold text-left">Owner</th>
          </tr>
        </thead>
        <tbody>
          {#each rooms as room}
            <tr>
              <td><Link to={"/room/" + room.id} class="text-blue-600 hover:text-blue-800">Join</Link></td>
              <td>{room.id}</td>
              <td>{room.title}</td>
              <td>{room.owner}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>