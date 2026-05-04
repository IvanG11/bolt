<template>
    <div class="app">
        <!-- ── Topbar ── -->
        <header class="topbar">
            <div class="brand">
                <span class="brand-bolt">⚡</span>
                <span class="brand-name">bolt</span>
            </div>
            <div class="brand-sep"></div>
            <nav class="nav-tabs">
                <button
                    class="nav-tab"
                    :class="{ active: view === 'projects' }"
                    @click="view = 'projects'"
                >
                    Projects
                </button>
                <button
                    class="nav-tab"
                    :class="{ active: view === 'config' }"
                    @click="openConfig"
                >
                    Config
                </button>
            </nav>
            <div v-if="view === 'projects'" class="search-wrap">
                <svg
                    class="search-icon"
                    width="13"
                    height="13"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <circle cx="11" cy="11" r="8" />
                    <line x1="21" y1="21" x2="16.65" y2="16.65" />
                </svg>
                <input
                    v-model="searchQuery"
                    class="search-in"
                    type="text"
                    placeholder="Filter projects..."
                />
                <button
                    v-if="searchQuery"
                    class="search-clear"
                    @click="searchQuery = ''"
                    title="Clear"
                >
                    <svg
                        width="10"
                        height="10"
                        viewBox="0 0 12 12"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                    >
                        <line x1="1" y1="1" x2="11" y2="11" />
                        <line x1="11" y1="1" x2="1" y2="11" />
                    </svg>
                </button>
            </div>
            <div class="topbar-right">
                <button
                    v-if="view === 'projects'"
                    @click="stopAllProjects"
                    class="icon-btn"
                    title="Stop all projects"
                    :disabled="stoppingAll"
                >
                    <svg
                        v-if="!stoppingAll"
                        width="12"
                        height="12"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                    >
                        <rect x="3" y="3" width="18" height="18" rx="2" />
                    </svg>
                    <svg
                        v-else
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2.5"
                        stroke-linecap="round"
                        class="spinning"
                    >
                        <path d="M21 2v6h-6" />
                        <path d="M3 12a9 9 0 0 1 15-6.7L21 8" />
                        <path d="M3 22v-6h6" />
                        <path d="M21 12a9 9 0 0 1-15 6.7L3 16" />
                    </svg>
                </button>
                <button
                    v-if="view === 'projects'"
                    @click="fetchProjects"
                    class="icon-btn"
                    title="Refresh"
                >
                    <svg
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2.5"
                        stroke-linecap="round"
                        :class="{ spinning: loading }"
                    >
                        <path d="M21 2v6h-6" />
                        <path d="M3 12a9 9 0 0 1 15-6.7L21 8" />
                        <path d="M3 22v-6h6" />
                        <path d="M21 12a9 9 0 0 1-15 6.7L3 16" />
                    </svg>
                </button>
            </div>
        </header>

        <!-- ── Error banner ── -->
        <transition name="slide-down">
            <div
                v-if="actionError"
                class="banner error-banner"
                @click="actionError = null"
            >
                <svg
                    width="13"
                    height="13"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                >
                    <circle cx="12" cy="12" r="10" />
                    <line x1="12" y1="8" x2="12" y2="12" />
                    <line x1="12" y1="16" x2="12.01" y2="16" />
                </svg>
                <span>{{ actionError }}</span>
                <button class="banner-x">&#10005;</button>
            </div>
        </transition>

        <!-- ── Workspace (main + drawer) ── -->
        <div class="workspace">
            <div class="main">
                <!-- Projects view -->
                <div v-show="view === 'projects'" class="scroll-area">
                    <div v-if="loading" class="empty-state">
                        <div class="spin-ring"></div>
                        <span>Connecting to Bolt...</span>
                    </div>
                    <div v-else-if="error" class="empty-state danger">
                        {{ error }}
                    </div>
                    <div
                        v-else-if="filteredProjects.length === 0"
                        class="empty-state"
                    >
                        <svg
                            width="28"
                            height="28"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="1.5"
                        >
                            <circle cx="11" cy="11" r="8" />
                            <line x1="21" y1="21" x2="16.65" y2="16.65" />
                        </svg>
                        <span>No projects match "{{ searchQuery }}"</span>
                    </div>
                    <div v-else-if="projects.length === 0" class="empty-state">
                        <svg
                            width="28"
                            height="28"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="1.5"
                        >
                            <rect x="3" y="3" width="7" height="7" rx="1" />
                            <rect x="14" y="3" width="7" height="7" rx="1" />
                            <rect x="3" y="14" width="7" height="7" rx="1" />
                            <rect x="14" y="14" width="7" height="7" rx="1" />
                        </svg>
                        <span>No projects found</span>
                    </div>
                    <div v-else class="grid">
                        <div
                            v-for="p in filteredProjects"
                            :key="p.name"
                            class="card"
                            :class="{
                                running: p.status === 'running',
                                selected: drawerName === p.name,
                            }"
                            @click="toggleDrawer(p.name)"
                        >
                            <div class="card-top">
                                <span
                                    class="pulse-dot"
                                    :class="p.status"
                                ></span>
                                <span class="card-name">{{ p.name }}</span>
                                <span class="card-status">{{ p.status }}</span>
                            </div>

                            <div
                                v-if="p.subdirs.length > 0"
                                class="card-subdirs"
                                @click.stop
                            >
                                <div
                                    v-for="s in p.subdirs"
                                    :key="s.name"
                                    class="subdir-row"
                                >
                                    <span
                                        class="subdir-badge"
                                        :class="s.status"
                                    >
                                        <span
                                            class="subdir-dot"
                                            :class="s.status"
                                        ></span>
                                        {{ s.name }}
                                    </span>
                                    <div class="subdir-btns">
                                        <button
                                            v-if="
                                                busySubdir[
                                                    p.name + '/' + s.name
                                                ]
                                            "
                                            class="svc-btn"
                                            disabled
                                        >
                                            <svg
                                                width="9"
                                                height="9"
                                                viewBox="0 0 24 24"
                                                fill="none"
                                                stroke="currentColor"
                                                stroke-width="2.5"
                                                class="spinning"
                                            >
                                                <path d="M21 2v6h-6" />
                                                <path
                                                    d="M3 12a9 9 0 0 1 15-6.7L21 8"
                                                />
                                                <path d="M3 22v-6h6" />
                                                <path
                                                    d="M21 12a9 9 0 0 1-15 6.7L3 16"
                                                />
                                            </svg>
                                        </button>
                                        <template v-else>
                                            <button
                                                v-if="s.status === 'stopped'"
                                                class="svc-btn start"
                                                @click="
                                                    doSubdirAction(
                                                        p.name,
                                                        s.name,
                                                        'start',
                                                    )
                                                "
                                                title="Start"
                                            >
                                                <svg
                                                    width="8"
                                                    height="9"
                                                    viewBox="0 0 10 12"
                                                    fill="currentColor"
                                                >
                                                    <polygon
                                                        points="0,0 10,6 0,12"
                                                    />
                                                </svg>
                                            </button>
                                            <template v-else>
                                                <button
                                                    class="svc-btn restart"
                                                    @click="
                                                        doSubdirAction(
                                                            p.name,
                                                            s.name,
                                                            'restart',
                                                        )
                                                    "
                                                    title="Restart"
                                                >
                                                    <svg
                                                        width="12"
                                                        height="12"
                                                        viewBox="0 0 24 24"
                                                        fill="none"
                                                        stroke="currentColor"
                                                        stroke-width="2.5"
                                                        stroke-linecap="round"
                                                        stroke-linejoin="round"
                                                    >
                                                        <path d="M1 4v6h6" />
                                                        <path
                                                            d="M3.51 15a9 9 0 1 0 .49-4.5L1 10"
                                                        />
                                                    </svg>
                                                </button>
                                                <button
                                                    class="svc-btn stop"
                                                    @click="
                                                        doSubdirAction(
                                                            p.name,
                                                            s.name,
                                                            'stop',
                                                        )
                                                    "
                                                    title="Stop"
                                                >
                                                    <svg
                                                        width="8"
                                                        height="8"
                                                        viewBox="0 0 10 10"
                                                        fill="currentColor"
                                                    >
                                                        <rect
                                                            width="10"
                                                            height="10"
                                                            rx="1.5"
                                                        />
                                                    </svg>
                                                </button>
                                            </template>
                                        </template>
                                    </div>
                                </div>
                            </div>

                            <div class="card-actions" @click.stop>
                                <button
                                    v-if="p.status === 'stopped'"
                                    class="act-btn green"
                                    :disabled="busy[p.name]"
                                    @click="doAction(p.name, 'start')"
                                >
                                    Start
                                </button>
                                <button
                                    v-if="p.status === 'running'"
                                    class="act-btn red"
                                    :disabled="busy[p.name]"
                                    @click="doAction(p.name, 'stop')"
                                >
                                    Stop
                                </button>
                                <button
                                    class="act-btn blue"
                                    :disabled="busy[p.name]"
                                    @click="doAction(p.name, 'restart')"
                                >
                                    Restart
                                </button>
                                <span v-if="busy[p.name]" class="busy-dots"
                                    ><span></span><span></span><span></span
                                ></span>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Config view -->
                <div v-show="view === 'config'" class="scroll-area">
                    <div v-if="cfgLoading" class="empty-state">
                        <div class="spin-ring"></div>
                        <span>Loading configuration...</span>
                    </div>
                    <div v-else class="cfg-body">
                        <transition name="slide-down">
                            <div
                                v-if="cfgSuccess"
                                class="banner success-banner"
                            >
                                <svg
                                    width="13"
                                    height="13"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2.5"
                                    stroke-linecap="round"
                                >
                                    <polyline points="20 6 9 17 4 12" />
                                </svg>
                                <span>{{ cfgSuccess }}</span>
                            </div>
                        </transition>

                        <section class="cfg-section">
                            <div class="cfg-section-head">
                                <div class="cfg-icon">
                                    <svg
                                        width="13"
                                        height="13"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        stroke-linecap="round"
                                    >
                                        <path
                                            d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
                                        />
                                    </svg>
                                </div>
                                <div>
                                    <h2 class="cfg-title">
                                        Projects Directory
                                    </h2>
                                    <p class="cfg-desc">
                                        Root folder where Bolt discovers Docker
                                        compose projects
                                    </p>
                                </div>
                            </div>
                            <div class="field-row">
                                <input
                                    v-model="dirDraft"
                                    class="text-in mono"
                                    type="text"
                                    placeholder="/home/user/projects"
                                    @keydown.enter="saveDir"
                                />
                                <button
                                    class="primary-btn"
                                    :disabled="
                                        savingDir ||
                                        !dirDraft.trim() ||
                                        dirDraft === cfg.projects_dir
                                    "
                                    @click="saveDir"
                                >
                                    {{ savingDir ? "Saving..." : "Save" }}
                                </button>
                            </div>
                        </section>

                        <div class="cfg-divider"></div>

                        <section class="cfg-section">
                            <div class="cfg-section-head">
                                <div class="cfg-icon">
                                    <svg
                                        width="13"
                                        height="13"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        stroke-linecap="round"
                                    >
                                        <circle cx="12" cy="12" r="10" />
                                        <line
                                            x1="4.93"
                                            y1="4.93"
                                            x2="19.07"
                                            y2="19.07"
                                        />
                                    </svg>
                                </div>
                                <div>
                                    <h2 class="cfg-title">Ignored Projects</h2>
                                    <p class="cfg-desc">
                                        These projects are hidden from list and
                                        switch commands
                                    </p>
                                </div>
                            </div>
                            <div class="chips-wrap">
                                <span
                                    v-if="cfg.ignore.length === 0"
                                    class="no-items"
                                    >No ignored projects</span
                                >
                                <span
                                    v-for="p in cfg.ignore"
                                    :key="p"
                                    class="chip"
                                >
                                    {{ p }}
                                    <button
                                        class="chip-x"
                                        :disabled="removingIgnore[p]"
                                        @click="removeIgnore(p)"
                                    >
                                        <svg
                                            width="9"
                                            height="9"
                                            viewBox="0 0 12 12"
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-width="2.5"
                                            stroke-linecap="round"
                                        >
                                            <line
                                                x1="1"
                                                y1="1"
                                                x2="11"
                                                y2="11"
                                            />
                                            <line
                                                x1="11"
                                                y1="1"
                                                x2="1"
                                                y2="11"
                                            />
                                        </svg>
                                    </button>
                                </span>
                            </div>
                            <div class="field-row" style="margin-top: 12px">
                                <input
                                    v-model="newIgnore"
                                    class="text-in"
                                    type="text"
                                    placeholder="project-name"
                                    @keydown.enter="addIgnore"
                                />
                                <button
                                    class="primary-btn"
                                    :disabled="
                                        addingIgnore || !newIgnore.trim()
                                    "
                                    @click="addIgnore"
                                >
                                    {{ addingIgnore ? "Adding..." : "Add" }}
                                </button>
                            </div>
                        </section>

                        <div class="cfg-divider"></div>

                        <section class="cfg-section">
                            <div class="cfg-section-head">
                                <div class="cfg-icon">
                                    <svg
                                        width="13"
                                        height="13"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        stroke-linecap="round"
                                    >
                                        <polyline
                                            points="22 12 18 12 15 21 9 3 6 12 2 12"
                                        />
                                    </svg>
                                </div>
                                <div>
                                    <h2 class="cfg-title">Project Subdirs</h2>
                                    <p class="cfg-desc">
                                        Restrict which subdirectories are
                                        managed per project
                                    </p>
                                </div>
                            </div>
                            <div class="subdirs-table">
                                <div class="tbl-head">
                                    <span>Project</span
                                    ><span>Subdirectories</span><span></span>
                                </div>
                                <div
                                    v-if="
                                        Object.keys(cfg.projects).length === 0
                                    "
                                    class="tbl-empty"
                                >
                                    No subdir configuration — add one below
                                </div>
                                <div
                                    v-for="(pc, name) in cfg.projects"
                                    :key="name"
                                    class="tbl-row"
                                >
                                    <span class="tbl-proj">{{ name }}</span>
                                    <div class="tbl-subdirs">
                                        <template
                                            v-if="
                                                editingSubdirs[name] !==
                                                undefined
                                            "
                                        >
                                            <input
                                                class="text-in small mono"
                                                v-model="editingSubdirs[name]"
                                                :placeholder="
                                                    pc.subdirs.join(',')
                                                "
                                                @keydown.enter="
                                                    saveSubdirs(name)
                                                "
                                            />
                                        </template>
                                        <template v-else>
                                            <span
                                                v-for="s in pc.subdirs"
                                                :key="s"
                                                class="subdir-pill"
                                                >{{ s }}</span
                                            >
                                        </template>
                                    </div>
                                    <div class="tbl-actions">
                                        <template
                                            v-if="
                                                editingSubdirs[name] !==
                                                undefined
                                            "
                                        >
                                            <button
                                                class="row-btn accent"
                                                @click="saveSubdirs(name)"
                                            >
                                                Save
                                            </button>
                                            <button
                                                class="row-btn muted"
                                                @click="cancelEdit(name)"
                                            >
                                                Cancel
                                            </button>
                                        </template>
                                        <template v-else>
                                            <button
                                                class="row-btn muted"
                                                @click="
                                                    startEdit(name, pc.subdirs)
                                                "
                                            >
                                                Edit
                                            </button>
                                            <button
                                                class="row-btn danger"
                                                @click="clearSubdirs(name)"
                                            >
                                                Clear
                                            </button>
                                        </template>
                                    </div>
                                </div>
                                <div class="tbl-row tbl-add">
                                    <input
                                        class="text-in small mono"
                                        v-model="newSubdirProject"
                                        placeholder="project-name"
                                    />
                                    <input
                                        class="text-in small mono"
                                        v-model="newSubdirList"
                                        placeholder="api,worker,db"
                                        @keydown.enter="addSubdirConfig"
                                    />
                                    <div class="tbl-actions">
                                        <button
                                            class="row-btn accent"
                                            :disabled="
                                                !newSubdirProject.trim() ||
                                                !newSubdirList.trim()
                                            "
                                            @click="addSubdirConfig"
                                        >
                                            Add
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </section>
                    </div>
                </div>
            </div>
            <!-- /main -->

            <!-- ── Project drawer (overlay) ── -->
            <transition name="drawer-slide">
                <div
                    v-if="drawerName"
                    class="drawer"
                    :style="{ width: drawerWidth + 'px' }"
                >
                    <!-- Resize handle -->
                    <div
                        class="resize-handle"
                        @mousedown.prevent="startResize"
                    ></div>

                    <!-- Drawer header -->
                    <div class="drawer-head">
                        <div class="drawer-title-row">
                            <span
                                class="pulse-dot"
                                :class="drawerProject?.status ?? 'stopped'"
                            ></span>
                            <span class="drawer-project-name">{{
                                drawerName
                            }}</span>
                            <span
                                class="drawer-status-badge"
                                :class="drawerProject?.status"
                                >{{ drawerProject?.status ?? "—" }}</span
                            >
                        </div>
                        <button class="icon-btn" @click="closeDrawer">
                            <svg
                                width="12"
                                height="12"
                                viewBox="0 0 12 12"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                            >
                                <line x1="1" y1="1" x2="11" y2="11" />
                                <line x1="11" y1="1" x2="1" y2="11" />
                            </svg>
                        </button>
                    </div>

                    <!-- Drawer scrollable body -->
                    <div class="drawer-body">
                        <!-- Location -->
                        <div class="dw-section">
                            <div class="dw-label">Location</div>
                            <div v-if="drawerInfoLoading" class="dw-loading">
                                <div class="spin-ring small"></div>
                                <span>Loading...</span>
                            </div>
                            <div v-else class="dw-path-row">
                                <span class="dw-path">{{
                                    drawerInfo.path || "—"
                                }}</span>
                                <button
                                    v-if="drawerInfo.path"
                                    class="copy-btn"
                                    @click="copyPath"
                                    :title="copied ? 'Copied!' : 'Copy path'"
                                >
                                    <svg
                                        v-if="!copied"
                                        width="12"
                                        height="12"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        stroke-linecap="round"
                                    >
                                        <rect
                                            x="9"
                                            y="9"
                                            width="13"
                                            height="13"
                                            rx="2"
                                        />
                                        <path
                                            d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"
                                        />
                                    </svg>
                                    <svg
                                        v-else
                                        width="12"
                                        height="12"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2.5"
                                        stroke-linecap="round"
                                    >
                                        <polyline points="20 6 9 17 4 12" />
                                    </svg>
                                </button>
                            </div>
                        </div>

                        <!-- Ports -->
                        <div
                            v-if="
                                !drawerInfoLoading &&
                                drawerInfo.ports?.length > 0
                            "
                            class="dw-section"
                        >
                            <div class="dw-label">Ports</div>
                            <div class="dw-ports">
                                <div
                                    v-for="p in drawerInfo.ports"
                                    :key="
                                        (p.host_port || p.url) +
                                        p.service +
                                        p.container_port
                                    "
                                    class="port-row"
                                >
                                    <a
                                        :href="
                                            p.url ||
                                            'http://localhost:' + p.host_port
                                        "
                                        target="_blank"
                                        rel="noopener"
                                        class="port-host"
                                    >
                                        {{
                                            p.url || "localhost:" + p.host_port
                                        }}
                                        <svg
                                            width="9"
                                            height="9"
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-width="2.5"
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                        >
                                            <path
                                                d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"
                                            />
                                            <polyline points="15 3 21 3 21 9" />
                                            <line
                                                x1="10"
                                                y1="14"
                                                x2="21"
                                                y2="3"
                                            />
                                        </svg>
                                    </a>
                                    <template v-if="p.host_port > 0">
                                        <svg
                                            width="10"
                                            height="10"
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-width="2.5"
                                            stroke-linecap="round"
                                            class="port-arrow-icon"
                                        >
                                            <line
                                                x1="5"
                                                y1="12"
                                                x2="19"
                                                y2="12"
                                            />
                                            <polyline
                                                points="12 5 19 12 12 19"
                                            />
                                        </svg>
                                        <span class="port-svc">{{
                                            p.service
                                        }}</span>
                                        <span class="port-container"
                                            >:{{ p.container_port }}</span
                                        >
                                        <span class="port-proto">{{
                                            p.protocol
                                        }}</span>
                                    </template>
                                    <template v-else>
                                        <svg
                                            width="10"
                                            height="10"
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-width="2.5"
                                            stroke-linecap="round"
                                            class="port-arrow-icon"
                                        >
                                            <line
                                                x1="5"
                                                y1="12"
                                                x2="19"
                                                y2="12"
                                            />
                                            <polyline
                                                points="12 5 19 12 12 19"
                                            />
                                        </svg>
                                        <span class="port-svc">{{
                                            p.service
                                        }}</span>
                                    </template>
                                </div>
                            </div>
                        </div>

                        <!-- Services -->
                        <div
                            v-if="drawerProject?.subdirs?.length > 0"
                            class="dw-section"
                        >
                            <div class="dw-label">Services</div>
                            <div class="dw-services">
                                <div
                                    v-for="s in drawerProject.subdirs"
                                    :key="s.name"
                                    class="dw-service-row"
                                    @click.stop
                                >
                                    <button
                                        v-if="s.status === 'running'"
                                        class="svc-btn shell"
                                        @click="openShell(drawerName, s.name)"
                                        title="Shell"
                                    >
                                        <svg
                                            width="12"
                                            height="12"
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-width="2.5"
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                        >
                                            <polyline points="4 17 10 11 4 5" />
                                            <line
                                                x1="12"
                                                y1="19"
                                                x2="20"
                                                y2="19"
                                            />
                                        </svg>
                                    </button>
                                    <span
                                        v-else
                                        class="svc-btn-placeholder"
                                    ></span>

                                    <span
                                        class="subdir-badge"
                                        :class="s.status"
                                    >
                                        <span
                                            class="subdir-dot"
                                            :class="s.status"
                                        ></span>
                                        {{ s.name }}
                                    </span>
                                    <div class="subdir-btns">
                                        <button
                                            v-if="
                                                busySubdir[
                                                    drawerName + '/' + s.name
                                                ]
                                            "
                                            class="svc-btn"
                                            disabled
                                        >
                                            <svg
                                                width="9"
                                                height="9"
                                                viewBox="0 0 24 24"
                                                fill="none"
                                                stroke="currentColor"
                                                stroke-width="2.5"
                                                class="spinning"
                                            >
                                                <path d="M21 2v6h-6" />
                                                <path
                                                    d="M3 12a9 9 0 0 1 15-6.7L21 8"
                                                />
                                                <path d="M3 22v-6h6" />
                                                <path
                                                    d="M21 12a9 9 0 0 1-15 6.7L3 16"
                                                />
                                            </svg>
                                        </button>
                                        <template v-else>
                                            <button
                                                v-if="s.status === 'stopped'"
                                                class="svc-btn start"
                                                @click="
                                                    doSubdirAction(
                                                        drawerName,
                                                        s.name,
                                                        'start',
                                                    )
                                                "
                                                title="Start"
                                            >
                                                <svg
                                                    width="8"
                                                    height="9"
                                                    viewBox="0 0 10 12"
                                                    fill="currentColor"
                                                >
                                                    <polygon
                                                        points="0,0 10,6 0,12"
                                                    />
                                                </svg>
                                            </button>
                                            <template v-else>
                                                <button
                                                    class="svc-btn restart"
                                                    @click="
                                                        doSubdirAction(
                                                            drawerName,
                                                            s.name,
                                                            'restart',
                                                        )
                                                    "
                                                    title="Restart"
                                                >
                                                    <svg
                                                        width="12"
                                                        height="12"
                                                        viewBox="0 0 24 24"
                                                        fill="none"
                                                        stroke="currentColor"
                                                        stroke-width="2.5"
                                                        stroke-linecap="round"
                                                        stroke-linejoin="round"
                                                    >
                                                        <path d="M1 4v6h6" />
                                                        <path
                                                            d="M3.51 15a9 9 0 1 0 .49-4.5L1 10"
                                                        />
                                                    </svg>
                                                </button>
                                                <button
                                                    class="svc-btn stop"
                                                    @click="
                                                        doSubdirAction(
                                                            drawerName,
                                                            s.name,
                                                            'stop',
                                                        )
                                                    "
                                                    title="Stop"
                                                >
                                                    <svg
                                                        width="8"
                                                        height="8"
                                                        viewBox="0 0 10 10"
                                                        fill="currentColor"
                                                    >
                                                        <rect
                                                            width="10"
                                                            height="10"
                                                            rx="1.5"
                                                        />
                                                    </svg>
                                                </button>
                                            </template>
                                        </template>
                                    </div>
                                </div>
                            </div>
                            <div class="dw-section-actions">
                                <button
                                    v-if="drawerProject?.status === 'stopped'"
                                    class="act-btn green"
                                    :disabled="busy[drawerName]"
                                    @click="doAction(drawerName, 'start')"
                                >
                                    Start
                                </button>
                                <button
                                    v-if="drawerProject?.status === 'running'"
                                    class="act-btn red"
                                    :disabled="busy[drawerName]"
                                    @click="doAction(drawerName, 'stop')"
                                >
                                    Stop
                                </button>
                                <button
                                    v-if="drawerProject?.status === 'running'"
                                    class="act-btn blue"
                                    :disabled="busy[drawerName]"
                                    @click="doAction(drawerName, 'restart')"
                                >
                                    Restart
                                </button>
                                <button
                                    class="act-btn muted"
                                    :disabled="
                                        busy[drawerName] || building[drawerName]
                                    "
                                    @click="doBuild(drawerName)"
                                    title="Rebuild Docker images"
                                >
                                    <svg
                                        width="11"
                                        height="11"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2.5"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        :class="{
                                            spinning: building[drawerName],
                                        }"
                                    >
                                        <path d="M21 2v6h-6" />
                                        <path d="M3 12a9 9 0 0 1 15-6.7L21 8" />
                                        <path d="M3 22v-6h6" />
                                        <path
                                            d="M21 12a9 9 0 0 1-15 6.7L3 16"
                                        />
                                    </svg>
                                    Build
                                </button>
                                <span
                                    v-if="
                                        busy[drawerName] || building[drawerName]
                                    "
                                    class="busy-dots"
                                    ><span></span><span></span><span></span
                                ></span>
                            </div>
                        </div>

                        <!-- Compose services (single-file projects without subdirs) -->
                        <div
                            v-if="drawerComposeServices.length > 0"
                            class="dw-section"
                        >
                            <div class="dw-label">Services</div>
                            <div class="dw-services">
                                <div
                                    v-for="s in drawerComposeServices"
                                    :key="s.name"
                                    class="dw-service-row"
                                    @click.stop
                                >
                                    <button
                                        class="svc-btn shell"
                                        :disabled="s.status !== 'running'"
                                        @click="openShell(drawerName, s.name)"
                                        title="Shell"
                                    >
                                        <svg
                                            width="12"
                                            height="12"
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke="currentColor"
                                            stroke-width="2.5"
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                        >
                                            <polyline points="4 17 10 11 4 5" />
                                            <line
                                                x1="12"
                                                y1="19"
                                                x2="20"
                                                y2="19"
                                            />
                                        </svg>
                                    </button>
                                    <span
                                        class="subdir-badge"
                                        :class="s.status"
                                    >
                                        <span
                                            class="subdir-dot"
                                            :class="s.status"
                                        ></span>
                                        {{ s.name }}
                                    </span>
                                </div>
                            </div>
                            <div class="dw-section-actions">
                                <button
                                    v-if="drawerProject?.status === 'stopped'"
                                    class="act-btn green"
                                    :disabled="busy[drawerName]"
                                    @click="doAction(drawerName, 'start')"
                                >
                                    Start
                                </button>
                                <button
                                    v-if="drawerProject?.status === 'running'"
                                    class="act-btn red"
                                    :disabled="busy[drawerName]"
                                    @click="doAction(drawerName, 'stop')"
                                >
                                    Stop
                                </button>
                                <button
                                    v-if="drawerProject?.status === 'running'"
                                    class="act-btn blue"
                                    :disabled="busy[drawerName]"
                                    @click="doAction(drawerName, 'restart')"
                                >
                                    Restart
                                </button>
                                <button
                                    class="act-btn muted"
                                    :disabled="
                                        busy[drawerName] || building[drawerName]
                                    "
                                    @click="doBuild(drawerName)"
                                    title="Rebuild Docker images"
                                >
                                    <svg
                                        width="11"
                                        height="11"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2.5"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        :class="{
                                            spinning: building[drawerName],
                                        }"
                                    >
                                        <path d="M21 2v6h-6" />
                                        <path d="M3 12a9 9 0 0 1 15-6.7L21 8" />
                                        <path d="M3 22v-6h6" />
                                        <path
                                            d="M21 12a9 9 0 0 1-15 6.7L3 16"
                                        />
                                    </svg>
                                    Build
                                </button>
                                <span
                                    v-if="
                                        busy[drawerName] || building[drawerName]
                                    "
                                    class="busy-dots"
                                    ><span></span><span></span><span></span
                                ></span>
                            </div>
                        </div>

                        <!-- Shell terminal -->
                        <div v-if="shellService" class="dw-shell">
                            <div class="dw-shell-header">
                                <span class="dw-label" style="margin-bottom: 0">
                                    <svg
                                        width="11"
                                        height="11"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2.5"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                    >
                                        <polyline points="4 17 10 11 4 5" />
                                        <line x1="12" y1="19" x2="20" y2="19" />
                                    </svg>
                                    {{ shellService }}
                                </span>
                                <button
                                    class="icon-btn"
                                    @click="closeShell"
                                    style="width: 22px; height: 22px"
                                >
                                    <svg
                                        width="10"
                                        height="10"
                                        viewBox="0 0 12 12"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        stroke-linecap="round"
                                    >
                                        <line x1="1" y1="1" x2="11" y2="11" />
                                        <line x1="11" y1="1" x2="1" y2="11" />
                                    </svg>
                                </button>
                            </div>
                            <div ref="termEl" class="dw-shell-term"></div>
                        </div>

                        <!-- Build output -->
                        <div v-if="buildLogs.length > 0" class="dw-build">
                            <div class="dw-build-header">
                                <span class="dw-label" style="margin-bottom: 0">
                                    <svg
                                        width="11"
                                        height="11"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2.5"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        :class="{
                                            spinning: building[drawerName],
                                        }"
                                    >
                                        <path d="M21 2v6h-6" />
                                        <path d="M3 12a9 9 0 0 1 15-6.7L21 8" />
                                        <path d="M3 22v-6h6" />
                                        <path
                                            d="M21 12a9 9 0 0 1-15 6.7L3 16"
                                        />
                                    </svg>
                                    Build{{
                                        building[drawerName]
                                            ? "ing..."
                                            : " output"
                                    }}
                                </span>
                                <button
                                    class="icon-btn"
                                    @click="buildLogs = []"
                                    style="width: 22px; height: 22px"
                                >
                                    <svg
                                        width="10"
                                        height="10"
                                        viewBox="0 0 12 12"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        stroke-linecap="round"
                                    >
                                        <line x1="1" y1="1" x2="11" y2="11" />
                                        <line x1="11" y1="1" x2="1" y2="11" />
                                    </svg>
                                </button>
                            </div>
                            <div
                                class="dw-log-body dw-build-body"
                                ref="buildRef"
                            >
                                <div
                                    v-for="(line, i) in buildLogs"
                                    :key="i"
                                    class="log-line"
                                >
                                    {{ line }}
                                </div>
                            </div>
                        </div>

                        <!-- Logs -->
                        <div
                            v-if="drawerProject?.status === 'running'"
                            class="dw-logs"
                            :class="{ open: logsOpen }"
                        >
                            <div class="dw-logs-header" @click="toggleLogs">
                                <span class="dw-label" style="margin-bottom: 0"
                                    >Logs</span
                                >
                                <div
                                    class="log-tabs"
                                    v-if="logsOpen"
                                    @click.stop
                                >
                                    <button
                                        v-for="s in drawerProject.subdirs"
                                        :key="s.name"
                                        class="log-tab"
                                        :class="{
                                            active: drawerSubdir === s.name,
                                        }"
                                        @click="switchDrawerSubdir(s.name)"
                                    >
                                        {{ s.name }}
                                    </button>
                                    <span
                                        v-if="!drawerProject.subdirs?.length"
                                        class="log-tab active"
                                        >all</span
                                    >
                                </div>
                                <svg
                                    class="logs-chevron"
                                    :class="{ rotated: logsOpen }"
                                    width="12"
                                    height="12"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2.5"
                                    stroke-linecap="round"
                                >
                                    <polyline points="6 9 12 15 18 9" />
                                </svg>
                            </div>
                            <div
                                v-if="logsOpen"
                                class="dw-log-body"
                                ref="logBody"
                            >
                                <div v-if="logs.length === 0" class="log-empty">
                                    Waiting for logs...
                                </div>
                                <div
                                    v-for="(line, i) in logs"
                                    :key="i"
                                    class="log-line"
                                >
                                    {{ line }}
                                </div>
                            </div>
                        </div>
                    </div>
                    <!-- /drawer-body -->
                </div>
            </transition>
        </div>
        <!-- /workspace -->
    </div>
</template>

<script setup>
import { ref, reactive, computed, watch, nextTick, onUnmounted } from "vue";
import { useShell } from "./composables/useShell.js";

// ── View ──────────────────────────────────────────────
const view = ref("projects");

// ── Search ────────────────────────────────────────────
const searchQuery = ref("");
const filteredProjects = computed(() => {
    const q = searchQuery.value.trim().toLowerCase();
    if (!q) return projects.value;
    return projects.value.filter((p) => p.name.toLowerCase().includes(q));
});

// ── Projects ──────────────────────────────────────────
const projects = ref([]);
const loading = ref(true);
const error = ref(null);
const busy = ref({});
const busySubdir = ref({});
const building = ref({});
const buildLogs = ref([]);
const buildRef = ref(null);
let buildSource = null;
const actionError = ref(null);
const stoppingAll = ref(false);

async function fetchProjects({ silent = false } = {}) {
    if (!silent) loading.value = true;
    error.value = null;
    try {
        const res = await fetch("/api/projects");
        const data = await res.json();
        projects.value = data.projects ?? [];
    } catch {
        error.value = 'Cannot reach Bolt API — is "bolt ui" running?';
    } finally {
        if (!silent) loading.value = false;
    }
}

async function doSubdirAction(project, subdir, action) {
    const key = `${project}/${subdir}`;
    busySubdir.value = { ...busySubdir.value, [key]: true };
    actionError.value = null;
    try {
        const res = await fetch(
            `/api/projects/${project}/subdirs/${subdir}/${action}`,
            { method: "POST" },
        );
        const data = await res.json();
        if (!data.ok) actionError.value = `${subdir}: ${data.error}`;
        await fetchProjects({ silent: true });
    } finally {
        busySubdir.value = { ...busySubdir.value, [key]: false };
    }
}

function doBuild(name) {
    if (buildSource) {
        buildSource.close();
        buildSource = null;
    }
    buildLogs.value = [];
    building.value = { ...building.value, [name]: true };
    actionError.value = null;

    buildSource = new EventSource(`/api/projects/${name}/build`);
    buildSource.onmessage = async (e) => {
        buildLogs.value = [...buildLogs.value, e.data];
        await nextTick();
        if (buildRef.value)
            buildRef.value.scrollTop = buildRef.value.scrollHeight;
    };
    buildSource.addEventListener("done", (e) => {
        buildSource.close();
        buildSource = null;
        building.value = { ...building.value, [name]: false };
        if (e.data === "error") actionError.value = "Build failed";
    });
    buildSource.onerror = () => {
        buildSource?.close();
        buildSource = null;
        building.value = { ...building.value, [name]: false };
    };
}

async function doAction(name, action) {
    busy.value = { ...busy.value, [name]: true };
    actionError.value = null;
    try {
        const res = await fetch(`/api/projects/${name}/${action}`, {
            method: "POST",
        });
        const data = await res.json();
        if (data && !data.ok) actionError.value = data.error;
        await fetchProjects({ silent: true });
        if (drawerName.value === name) fetchDrawerInfo(name);
    } finally {
        busy.value = { ...busy.value, [name]: false };
    }
}

async function stopAllProjects() {
    stoppingAll.value = true;
    actionError.value = null;
    try {
        const res = await fetch("/api/projects/stop-all", { method: "POST" });
        const data = await res.json();
        if (data && !data.ok) actionError.value = data.error;
        await fetchProjects({ silent: true });
    } catch {
        actionError.value = "Failed to stop all projects";
    } finally {
        stoppingAll.value = false;
    }
}

// ── Drawer ────────────────────────────────────────────
const drawerName = ref(null);
const drawerInfo = ref({ path: "", ports: [] });
const drawerInfoLoading = ref(false);
const drawerComposeServices = ref([]);
const drawerSubdir = ref(null);
const drawerWidth = ref(900);
const logsOpen = ref(false);
const copied = ref(false);
const logs = ref([]);
const logBody = ref(null);
let eventSource = null;

const MIN_DRAWER_WIDTH = 260;
const MAX_DRAWER_WIDTH = 1400;

const drawerProject = computed(
    () => projects.value.find((p) => p.name === drawerName.value) ?? null,
);

function connectLogs(name, subdir) {
    if (eventSource) {
        eventSource.close();
        eventSource = null;
    }
    logs.value = [];
    const url = subdir
        ? `/api/projects/${name}/logs?subdir=${subdir}`
        : `/api/projects/${name}/logs`;
    eventSource = new EventSource(url);
    eventSource.onmessage = (e) => {
        logs.value = [...logs.value, e.data];
        if (logs.value.length > 1000) logs.value = logs.value.slice(-900);
        nextTick(() => {
            if (logBody.value)
                logBody.value.scrollTop = logBody.value.scrollHeight;
        });
    };
    eventSource.onerror = () => {
        if (eventSource?.readyState === EventSource.CLOSED) {
            eventSource = null;
            logsOpen.value = false;
        }
    };
}

function stopLogs() {
    if (eventSource) {
        eventSource.close();
        eventSource = null;
    }
    logs.value = [];
}

async function fetchDrawerInfo(name) {
    drawerInfoLoading.value = true;
    drawerInfo.value = { path: "", ports: [] };
    try {
        const res = await fetch(`/api/projects/${name}/info`);
        const data = await res.json();
        drawerInfo.value = { path: data.path ?? "", ports: data.ports ?? [] };
    } finally {
        drawerInfoLoading.value = false;
    }
}

function toggleDrawer(name) {
    if (drawerName.value === name) {
        closeDrawer();
        return;
    }
    stopLogs();
    closeShell();
    drawerName.value = name;
    drawerSubdir.value = null;
    drawerComposeServices.value = [];
    logsOpen.value = false;
    fetchDrawerInfo(name);
    const proj = projects.value.find((p) => p.name === name);
    if (!proj?.subdirs?.length) fetchComposeServices(name);
}

function toggleLogs() {
    if (!logsOpen.value) {
        logsOpen.value = true;
        const proj = drawerProject.value;
        const first = proj?.subdirs?.[0]?.name ?? null;
        drawerSubdir.value = first;
        connectLogs(drawerName.value, first);
    } else {
        logsOpen.value = false;
        stopLogs();
    }
}

async function fetchComposeServices(name) {
    const res = await fetch(`/api/projects/${name}/services`);
    const data = await res.json();
    drawerComposeServices.value = data.services ?? [];
}

function closeDrawer() {
    drawerName.value = null;
    drawerSubdir.value = null;
    drawerInfo.value = { path: "", ports: [] };
    drawerComposeServices.value = [];
    logsOpen.value = false;
    buildLogs.value = [];
    if (buildSource) {
        buildSource.close();
        buildSource = null;
    }
    stopLogs();
    closeShell();
}

function switchDrawerSubdir(subdir) {
    drawerSubdir.value = subdir;
    if (logsOpen.value) connectLogs(drawerName.value, subdir);
}

async function copyPath() {
    if (!drawerInfo.value.path) return;
    await navigator.clipboard.writeText(drawerInfo.value.path);
    copied.value = true;
    setTimeout(() => {
        copied.value = false;
    }, 2000);
}

// Stop logs if project goes down while logs are open
watch(projects, (updated) => {
    if (!drawerName.value) return;
    const proj = updated.find((p) => p.name === drawerName.value);
    if (proj && proj.status !== "running" && eventSource) {
        stopLogs();
        logsOpen.value = false;
    }
});

// ── Resize ────────────────────────────────────────────
function startResize(e) {
    const onMove = (e) => {
        const newWidth = window.innerWidth - e.clientX;
        drawerWidth.value = Math.max(
            MIN_DRAWER_WIDTH,
            Math.min(MAX_DRAWER_WIDTH, newWidth),
        );
    };
    const onUp = () => {
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
    };
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    document.addEventListener("mousemove", onMove);
    document.addEventListener("mouseup", onUp);
}

// ── Config ────────────────────────────────────────────
const cfgLoading = ref(false);
const cfgLoaded = ref(false);
const cfgSuccess = ref(null);
const cfg = ref({ projects_dir: "", ignore: [], projects: {} });
const dirDraft = ref("");
const savingDir = ref(false);
const newIgnore = ref("");
const addingIgnore = ref(false);
const removingIgnore = reactive({});
const newSubdirProject = ref("");
const newSubdirList = ref("");
const editingSubdirs = reactive({});

async function fetchConfig() {
    cfgLoading.value = true;
    try {
        const res = await fetch("/api/config");
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const data = await res.json();
        cfg.value = {
            projects_dir: data.projects_dir ?? "",
            ignore: data.ignore ?? [],
            projects: data.projects ?? {},
        };
        dirDraft.value = cfg.value.projects_dir;
        cfgLoaded.value = true;
    } catch (e) {
        actionError.value = `Could not load config: ${e.message}`;
    } finally {
        cfgLoading.value = false;
    }
}

function flash(msg) {
    cfgSuccess.value = msg;
    setTimeout(() => {
        cfgSuccess.value = null;
    }, 3000);
}

async function saveDir() {
    const path = dirDraft.value.trim();
    if (!path || path === cfg.value.projects_dir) return;
    savingDir.value = true;
    try {
        const res = await fetch("/api/config/dir", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ path }),
        });
        const data = await res.json();
        if (data.ok) {
            cfg.value.projects_dir = path;
            flash("Projects directory saved");
            fetchProjects({ silent: true });
        } else actionError.value = data.error;
    } finally {
        savingDir.value = false;
    }
}

async function addIgnore() {
    const project = newIgnore.value.trim();
    if (!project) return;
    addingIgnore.value = true;
    try {
        const res = await fetch("/api/config/ignore", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ project }),
        });
        const data = await res.json();
        if (data.ok) {
            cfg.value.ignore = [...cfg.value.ignore, project];
            newIgnore.value = "";
            flash(`"${project}" ignored`);
            fetchProjects({ silent: true });
        } else actionError.value = data.error;
    } finally {
        addingIgnore.value = false;
    }
}

async function removeIgnore(project) {
    removingIgnore[project] = true;
    try {
        const res = await fetch(
            `/api/config/ignore/${encodeURIComponent(project)}`,
            { method: "DELETE" },
        );
        const data = await res.json();
        if (data.ok) {
            cfg.value.ignore = cfg.value.ignore.filter((x) => x !== project);
            flash(`"${project}" removed`);
            fetchProjects({ silent: true });
        } else actionError.value = data.error;
    } finally {
        delete removingIgnore[project];
    }
}

function startEdit(name, subdirs) {
    editingSubdirs[name] = subdirs.join(",");
}
function cancelEdit(name) {
    delete editingSubdirs[name];
}

async function saveSubdirs(name) {
    const subdirs = (editingSubdirs[name] ?? "")
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
    const res = await fetch(
        `/api/config/projects/${encodeURIComponent(name)}/subdirs`,
        {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ subdirs }),
        },
    );
    const data = await res.json();
    if (data.ok) {
        cfg.value.projects = { ...cfg.value.projects, [name]: { subdirs } };
        delete editingSubdirs[name];
        flash(`Subdirs saved for "${name}"`);
    } else actionError.value = data.error;
}

async function clearSubdirs(name) {
    const res = await fetch(
        `/api/config/projects/${encodeURIComponent(name)}/subdirs`,
        { method: "DELETE" },
    );
    const data = await res.json();
    if (data.ok) {
        const p = { ...cfg.value.projects };
        delete p[name];
        cfg.value.projects = p;
        flash(`Subdirs cleared for "${name}"`);
    } else actionError.value = data.error;
}

async function addSubdirConfig() {
    const name = newSubdirProject.value.trim();
    const subdirs = newSubdirList.value
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
    if (!name || !subdirs.length) return;
    const res = await fetch(
        `/api/config/projects/${encodeURIComponent(name)}/subdirs`,
        {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ subdirs }),
        },
    );
    const data = await res.json();
    if (data.ok) {
        cfg.value.projects = { ...cfg.value.projects, [name]: { subdirs } };
        newSubdirProject.value = "";
        newSubdirList.value = "";
        flash(`Subdirs set for "${name}"`);
    } else actionError.value = data.error;
}

function openConfig() {
    view.value = "config";
    if (!cfgLoaded.value) fetchConfig();
}

// ── Shell ─────────────────────────────────────────────
const { shellService, termEl, openShell, closeShell } = useShell();

// ── Keyboard shortcuts ────────────────────────────────
document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && drawerName.value) closeDrawer();
});

// ── Init ──────────────────────────────────────────────
fetchProjects();
let pollInterval = setInterval(() => fetchProjects({ silent: true }), 15000);
onUnmounted(() => {
    stopLogs();
    clearInterval(pollInterval);
});

document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
        clearInterval(pollInterval);
    } else {
        fetchProjects({ silent: true });
        pollInterval = setInterval(
            () => fetchProjects({ silent: true }),
            15000,
        );
    }
});
</script>
