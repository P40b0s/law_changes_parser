<template lang="pug">
    svg(xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" :width="props.width" :height="props.height" viewBox="0 0 256 256" xml:space="preserve")
        g(style="stroke: none; stroke-width: 0; stroke-dasharray: none; stroke-linecap: butt; stroke-linejoin: miter; stroke-miterlimit: 10; fill: none; fill-rule: nonzero; opacity: 1;" transform="translate(1.4065934065934016 1.4065934065934016) scale(2.81 2.81)")
            path.p1(d="M 45 90 C 20.187 90 0 69.813 0 45 C 0 20.187 20.187 0 45 0 c 24.813 0 45 20.187 45 45 C 90 69.813 69.813 90 45 90 z"  transform=" matrix(1 0 0 1 0 0) " stroke-linecap="round")
            path(d="M 31.671 69.566 c -1.821 0 -3.533 -0.709 -4.821 -1.997 l -5.42 -5.42 c -2.658 -2.658 -2.658 -6.983 0 -9.642 l 15.794 -15.793 c 2.658 -2.659 6.983 -2.659 9.642 0 l 1.296 1.296 c 1.171 1.172 1.171 3.071 -0.001 4.243 c -1.173 1.171 -3.071 1.171 -4.243 0 l -1.295 -1.296 c -0.318 -0.318 -0.837 -0.318 -1.157 0 L 25.673 56.75 c -0.318 0.319 -0.318 0.838 0 1.157 l 5.42 5.42 c 0.198 0.198 0.424 0.239 0.578 0.239 c 0.155 0 0.38 -0.041 0.578 -0.239 l 4.469 -4.47 c 1.172 -1.172 3.071 -1.172 4.243 0 c 1.172 1.171 1.172 3.07 0 4.242 l -4.469 4.47 C 35.205 68.857 33.493 69.566 31.671 69.566 z" style="stroke: none; stroke-width: 1; stroke-dasharray: none; stroke-linecap: butt; stroke-linejoin: miter; stroke-miterlimit: 10; fill: rgb(255,255,255); fill-rule: nonzero; opacity: 1;" transform=" matrix(1 0 0 1 0 0) " stroke-linecap="round")
            path(d="M 45.955 55.279 c -1.746 0 -3.491 -0.664 -4.82 -1.992 l -1.296 -1.297 c -1.172 -1.171 -1.172 -3.071 0 -4.242 c 1.171 -1.172 3.071 -1.172 4.242 0 l 1.296 1.296 c 0.319 0.318 0.838 0.319 1.157 0 L 62.327 33.25 c 0.198 -0.198 0.239 -0.424 0.239 -0.578 c 0 -0.155 -0.041 -0.38 -0.239 -0.578 l -5.42 -5.421 c -0.319 -0.319 -0.839 -0.318 -1.157 0 l -4.47 4.469 c -1.172 1.171 -3.07 1.171 -4.242 0 c -1.172 -1.171 -1.172 -3.071 0 -4.243 l 4.47 -4.469 c 2.657 -2.657 6.982 -2.658 9.642 0 l 5.42 5.421 c 1.288 1.287 1.997 2.999 1.997 4.821 c 0 1.821 -0.709 3.533 -1.997 4.821 L 50.775 53.286 C 49.447 54.615 47.701 55.279 45.955 55.279 z" style="stroke: none; stroke-width: 1; stroke-dasharray: none; stroke-linecap: butt; stroke-linejoin: miter; stroke-miterlimit: 10; fill: rgb(255,255,255); fill-rule: nonzero; opacity: 1;" transform=" matrix(1 0 0 1 0 0) " stroke-linecap="round")
    </template>
        
    <script lang="ts">
    import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs } from 'vue';
    
    </script>
    <script lang="ts" async setup>
    const props = defineProps<{
        width: number,
        height: number,
        color: string
    }>();
    
    
    </script>
        
    <style lang="scss" scoped>
    .p1
    {
        stroke: none;
        stroke-width: 1;
        stroke-dasharray: none;
        stroke-linecap: butt;
        stroke-linejoin: miter;
        stroke-miterlimit: 10;
        fill: v-bind(color);
        fill-rule: nonzero;
        opacity: 1;
    }
    </style>