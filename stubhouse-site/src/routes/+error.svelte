<script lang="ts">
  import { page } from '$app/state';
  import Button from '$lib/components/Button.svelte';
</script>

<section class="wrap container">
  <p class="eyebrow">{page.status}</p>
  <h1 class="display-2">This URL is not a mock route.</h1>
  <p class="body-lg prose-width">The page is gone or never shipped. Check the path or return home.</p>
  <div class="actions">
    <Button href="/" variant="primary" size="md">{#snippet children()}Back home{/snippet}</Button>
  </div>
</section>

<style>
  .wrap {
    padding-block: 120px;
  }

  .actions {
    margin-top: 24px;
  }
</style>
