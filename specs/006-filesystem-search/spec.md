# Feature Specification: v0.5 Filesystem Search

**Feature Branch**: `[006-filesystem-search]`

**Created**: 2026-08-09

**Status**: Draft

**Input**: User description: "v0.5 Search from ROADMAP.md: filesystem search, relevant document loading"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Find relevant documents in a growing workspace (Priority: P1)

As a workspace user with a large amount of base and project documentation, I can search my workspace for documents relevant to a topic so I don't have to remember exact file names or manually browse folders.

**Why this priority**: This is the core value of v0.5. Today the Context Builder (v0.4) can only include whatever fits under its fixed source/byte caps, in a fixed priority order (project > base > session) — as a workspace accumulates documentation, that stops being "relevant" and starts being arbitrary. Search is what makes "relevant" actually mean something.

**Independent Test**: Can be fully tested by creating a workspace with dozens of markdown documents across base and project knowledge, searching for a topic covered by only a few of them, and confirming the returned documents are the ones whose content relates to that topic rather than an arbitrary subset.

**Acceptance Scenarios**:

1. **Given** a project with more markdown documents than the Context Builder's caps allow, **When** the user searches for a specific topic, **Then** the returned results are the documents whose content relates to that topic rather than an arbitrary subset chosen by file order.
2. **Given** a search query with no matches anywhere in the accessible workspace, **When** the user searches, **Then** the system clearly reports that no matches were found instead of returning unrelated documents or failing with an error.
3. **Given** a search query with many matching documents, **When** results are returned, **Then** the most relevant matches are presented ahead of weaker matches.

---

### User Story 2 - Focus session context on the current task (Priority: P2)

As a workspace user, I can have my session's assembled context reflect the topic I'm actually working on, so the AI receives the most relevant material for my current question instead of a static "everything that fits under the cap" snapshot.

**Why this priority**: This connects search to the existing Context Builder (v0.4) and delivers the "relevant document loading" half of the roadmap goal. Without it, search would be no more than a standalone file-finder disconnected from what the AI actually sees.

**Independent Test**: Can be fully tested by starting a session with a task-specific query in a workspace whose documentation exceeds the Context Builder's caps, and confirming the assembled context is composed of the documents search identified as relevant to that query rather than the first documents encountered in priority order.

**Acceptance Scenarios**:

1. **Given** a workspace whose total base and project documentation exceeds the Context Builder's existing size caps, **When** the user starts a session or asks a question, **Then** the assembled context is selected using search relevance rather than truncated in file-encounter order.
2. **Given** a workspace small enough that all documentation already fits under the existing caps, **When** the user starts a session, **Then** the assembled context still includes all previously-included Base and Project material (search does not remove content that already fit).

---

### User Story 3 - See what a search found before it's used (Priority: P3)

As a workspace user, I can see which documents a search matched and a snippet of why, so I can judge relevance and trust what material is being made available to the AI.

**Why this priority**: This matches the transparency posture already established by the permission model (users can always view what's granted and why) and by the Context Builder's requirement to make context scope understandable before the AI responds. It is lower priority than Stories 1 and 2 because search is still useful without a dedicated inspection view, just harder to trust.

**Independent Test**: Can be fully tested by running a search and confirming the output lists each matched file with a relevance-indicating snippet, independent of whether that search is also feeding a live session.

**Acceptance Scenarios**:

1. **Given** a completed search, **When** the user reviews the results, **Then** each result shows the matched file's location and a snippet indicating why it matched.
2. **Given** a search run outside of an active session, **When** results are returned, **Then** the results are usable purely as a lookup tool without requiring the user to start a session.

### Edge Cases

- A search query matches zero documents anywhere the user currently has permission to see.
- A search query is very generic and matches most of the workspace's documents.
- The workspace contains documents outside any path the user currently has Default or Temporary permission for — these must never appear in results.
- A previously-matched document is deleted or becomes unavailable between search and context assembly.
- The workspace contains non-markdown files that the Context Builder does not otherwise load.
- A search runs in a workspace with no documentation at all (freshly initialized workspace).
- The active project changes between when a search runs and when its results would be used.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST let users search the markdown content of their accessible workspace (Base knowledge and the active Project's knowledge) using freeform text.
- **FR-002**: The system MUST restrict search results to files within paths the user currently holds Default or Temporary permission for, consistent with the existing permission model; matches outside granted paths MUST NOT appear in results.
- **FR-003**: The system MUST rank search results by relevance to the query rather than returning them in an arbitrary or purely alphabetical order.
- **FR-004**: The system MUST report clearly when a search produces no matches, distinct from an error condition.
- **FR-005**: Each search result MUST include enough information (file location and a matched snippet) for the user to judge why it was considered relevant.
- **FR-006**: The system MUST use search automatically as part of context assembly — whenever a session starts or a question is asked, search results (not just file-encounter order) determine which Base and Project documents the Context Builder includes.
- **FR-007**: When the workspace's total available documentation exceeds the Context Builder's existing source/byte caps, the system MUST use search relevance — rather than file-encounter order — to decide which documents are included in the assembled context.
- **FR-008**: The system MUST continue to honor the Context Builder's existing priority order (project > base > session) and size caps when combining search-selected documents into a context package.
- **FR-009**: The system MUST treat a document that was previously matched but has since become unavailable the same way the Context Builder already treats missing context sources (skip gracefully, continue with what's available).
- **FR-010**: The system MUST NOT require the user to start a full AI session in order to run a search and view its results.

### Key Entities *(include if feature involves data)*

- **Search Query**: The freeform text a user provides to find relevant documents; scoped to the workspace paths the user currently has permission to access.
- **Search Result**: A single matched document, carrying its file location, a relevance ranking, and a snippet showing why it matched the query.
- **Relevance Score**: An internal ranking value used to order Search Results and to decide which documents the Context Builder includes when available documentation exceeds its caps.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In a workspace with more documents than the Context Builder's caps allow, users searching for a topic find the documents that actually cover it, not an arbitrary subset, in at least 9 out of 10 representative queries.
- **SC-002**: Users can tell within seconds of viewing search results whether a query found relevant material or nothing at all, without needing to open any of the returned files.
- **SC-003**: A search over a workspace with hundreds of documents returns results in under 2 seconds.
- **SC-004**: When a workspace's documentation grows well beyond what previously fit under the Context Builder's caps, the assembled session context remains focused on the current task instead of becoming visibly less relevant as the workspace grows.
- **SC-005**: Zero search results are ever returned for a path the user does not currently hold permission for, verified across all tested permission configurations.

## Assumptions

- Search operates only over the markdown content already collected by the existing `collect_markdown_files` mechanism (Base + Project knowledge); non-markdown files remain out of scope for v0.5, consistent with the Context Builder.
- "Filesystem search" per the architecture's Search Abstraction (Phase 1) means simple, local keyword/text matching over file contents — not embeddings, vector search, or RAG, which are explicitly later phases.
- Search always respects the existing permission model; no new permission category is introduced by this feature.
- Search is workspace-local and offline, consistent with the project's local-first, no-cloud-dependency principle.
- A freshly initialized workspace with no documentation yields an empty, clearly-labeled "no documents to search" result rather than an error.
