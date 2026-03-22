<script lang="ts">
  import { cn } from "$lib/utils";
  import buyuIcon from "../../../../buyu_Icon.png";

  let {
    size = 40,
    alt = "BuYu",
    className = "",
    rounded = "xl"
  }: {
    size?: number;
    alt?: string;
    className?: string;
    rounded?: "md" | "lg" | "xl" | "full";
  } = $props();

  const roundedClass = $derived(
    rounded === "full"
      ? "rounded-full"
      : rounded === "md"
        ? "rounded-[var(--radius-md)]"
        : rounded === "lg"
          ? "rounded-[var(--radius-lg)]"
          : "rounded-[var(--radius-xl)]"
  );
</script>

<img
  src={buyuIcon}
  {alt}
  class={cn("select-none object-cover shadow-sm", roundedClass, className)}
  style={`width:${size}px;height:${size}px;`}
  draggable="false"
/>
