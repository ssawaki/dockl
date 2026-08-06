<script lang="ts">
  /**
   * Dev-only. Puts the containers page on screen with fixed data, for the screenshots in
   * README.md.
   *
   * The real page can't be used for that: it shows whatever containers happen to be
   * running, which on a work machine means real project names in a public image. This
   * renders the same components with data that's safe to publish.
   *
   * Same components, not a mockup — ContainerMasterList and ContainerDetailPanel are the
   * ones the app actually uses, so a screenshot taken here can't drift from what users
   * see. `src/routes/dev-*` is moved aside by scripts/build-without-dev-routes.mjs, so
   * none of this reaches a production build.
   */
  import MasterDetail from "$lib/components/layout/MasterDetail.svelte";
  import PageHeader from "$lib/components/layout/PageHeader.svelte";
  import ContainerMasterList from "$lib/components/containers/ContainerMasterList.svelte";
  import ContainerDetailPanel from "$lib/components/containers/ContainerDetailPanel.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { t } from "$lib/stores/i18n";
  import arrowClockwiseIcon from "@fluentui/svg-icons/icons/arrow_clockwise_16_regular.svg?raw";
  import type { ContainerSummary, ContainerDetail, DetailTabId } from "$lib/types";

  const COMPOSE_LABELS = {
    "com.docker.compose.project": "shopfront",
    "com.docker.compose.project.config_files": "/home/dev/shopfront/compose.yaml",
    "com.docker.compose.service": "",
  };

  // Deliberately ordinary: a Compose project of three services plus two standalone
  // containers, one of them stopped, so the grouping, the status dots and the "stopped"
  // section all have something to show.
  const containers: ContainerSummary[] = [
    {
      id: "3f2a1b9c4d5e",
      names: ["shopfront-web-1"],
      image: "nginx:1.27-alpine",
      state: "running",
      status: "Up 2 hours",
      ports: "0.0.0.0:8080->80/tcp",
      labels: { ...COMPOSE_LABELS, "com.docker.compose.service": "web" },
    },
    {
      id: "7c81d3e0a29b",
      names: ["shopfront-api-1"],
      image: "shopfront/api:1.4.2",
      state: "running",
      status: "Up 2 hours (healthy)",
      ports: "0.0.0.0:3000->3000/tcp",
      labels: { ...COMPOSE_LABELS, "com.docker.compose.service": "api" },
    },
    {
      id: "b40f6a1c8d73",
      names: ["shopfront-db-1"],
      image: "postgres:17-alpine",
      state: "running",
      status: "Up 2 hours",
      ports: "5432/tcp",
      labels: { ...COMPOSE_LABELS, "com.docker.compose.service": "db" },
    },
    {
      id: "e59b2f7a0c14",
      names: ["redis-cache"],
      image: "redis:7-alpine",
      state: "running",
      status: "Up 6 days",
      ports: "0.0.0.0:6379->6379/tcp",
      labels: {},
    },
    {
      id: "1d8e4c2b6f90",
      names: ["mailpit"],
      image: "axllent/mailpit:latest",
      state: "exited",
      status: "Exited (0) 3 days ago",
      ports: "",
      labels: {},
    },
  ];

  const apiDetail: ContainerDetail = {
    id: "7c81d3e0a29bf4c6e1a85d3079b2c418e6a7f90d5b3c821e4f7a06d9c3b5e128",
    name: "shopfront-api-1",
    image: "shopfront/api:1.4.2",
    status: "running",
    health: "healthy",
    created: "2026-08-04T09:12:44Z",
    ip_address: "172.19.0.3",
    ports: [
      { host_ip: "0.0.0.0", host_port: "3000", container_port: "3000", protocol: "tcp" },
      { host_ip: "0.0.0.0", host_port: "9229", container_port: "9229", protocol: "tcp" },
    ],
    mounts: [
      {
        mount_type: "bind",
        source: "/home/dev/shopfront/api",
        destination: "/usr/src/app",
      },
      {
        mount_type: "volume",
        source: "shopfront_node_modules",
        destination: "/usr/src/app/node_modules",
      },
    ],
    labels: {
      "com.docker.compose.project": "shopfront",
      "com.docker.compose.service": "api",
      "org.opencontainers.image.source": "https://github.com/example/shopfront",
      "org.opencontainers.image.version": "1.4.2",
    },
    cpu_limit_cores: 2,
    restart_policy: "unless-stopped",
  };

  let selectedId = $state<string | null>("7c81d3e0a29b");
  let selectedProject = $state<string | null>(null);
  let activeTab = $state<DetailTabId>("info");
</script>

<div class="page-view">
  <PageHeader title={$t("nav.containers")}>
    <!-- Inert on purpose: there's nothing to refresh here, it's only present so the header
         matches the real page in a screenshot. -->
    <fluent-button
      appearance="outline"
      icon-only
      tabindex="-1"
      title={$t("common.refresh")}
      aria-label={$t("common.refresh")}
    >
      <Icon svg={arrowClockwiseIcon} size={14} />
    </fluent-button>
  </PageHeader>

  <MasterDetail>
    {#snippet list()}
      <ContainerMasterList
        {containers}
        bind:selectedId
        bind:selectedProject
        onAction={() => {}}
        onComposeAction={() => {}}
      />
    {/snippet}
    {#snippet detail()}
      <ContainerDetailPanel
        containerId={selectedId}
        liveState="running"
        detailOverride={apiDetail}
        bind:activeTab
      />
    {/snippet}
  </MasterDetail>
</div>
