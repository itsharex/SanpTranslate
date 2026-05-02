/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

import "vue-router";

declare module "vue-router" {
  export interface RouteMeta {}
}

import "vue-i18n";

declare module "vue-i18n" {
  export interface DefineLocaleMessage {}
}
