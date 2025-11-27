<script>
    import LoaderCircle from "@lucide/svelte/icons/loader-circle"
</script>

<div class="animate w-fit h-fit">
    <LoaderCircle class="h-10 w-10" />
</div>

<style>
    @keyframes spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }

    .animate {
        animation: spin 1s linear infinite;
    }
</style>
