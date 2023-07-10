<script lang="ts">
  import GoldenLayout from "svelte-golden-layout";
  import type {JsonValue, ResolvedLayoutConfig, VirtualLayout, LayoutConfig} from "golden-layout";
  import Test from "./Test.svelte";
  import type {ComponentType} from "svelte";

  const components: Record<string, ComponentType> = {Test};
  let goldenLayout: VirtualLayout;
  const layout: LayoutConfig = {
    root: {
      type: "row",
      content: [
        {
          type: "component",
          componentType: "Test",
          componentState: {
            someProp: 1,
            anotherProp: 1,
          },
        },
        {
          type: "component",
          componentType: "Test",
        },
      ],
    },
    settings: {
      showPopoutIcon: false,
      showMaximiseIcon: false,
    },
  };

  function castComponentState(componentState: JsonValue | undefined): object | undefined {
    return componentState as object | undefined;
  }
</script>

<nav>
  <a class="link" href="/">home</a>
  <a class="link" href="/example">example</a>
</nav>

<h1 class="text-3xl text-purple-500 font-bold underline">example</h1>
<p>this is the example page.</p>

<button class="btn">Button</button>
<button class="btn btn-neutral">Neutral</button>
<button class="btn btn-primary">Button</button>
<button class="btn btn-secondary">Button</button>
<button class="btn btn-accent">Button</button>
<button class="btn btn-ghost">Button</button>
<button class="btn btn-link">Button</button>

<div class="layout-container">
  <GoldenLayout config={layout} bind:goldenLayout let:componentType let:componentState>
    <svelte:component this={components[componentType]} {...castComponentState(componentState)} />
  </GoldenLayout>
</div>

<style>
  .layout-container {
    width: 100%;
    height: 100%;

    border: 1px solid black;
  }
</style>
