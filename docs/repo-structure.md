# Repository Structure
```
CAPPING2025/
├── .github/
│   ├── ISSUE_TEMPLATE/
│   │   ├── epic.yml
│   │   ├── misc.yml
│   │   └── story.yml
│   └── workflows/
│       ├── ci.yml
│       └── issue-manager.yml
├── .sqlx/
├── docs/
│   ├── frontend-codecov/
│   │   ├── src/
│   │   │   └── helpers/
│   │   │       ├── account.ts.html
│   │   │       ├── index.html
│   │   │       └── itinerary.ts.html
│   │   ├── tests/
│   │   │   ├── index.html
│   │   │   └── testApi.ts.html
│   │   ├── base.css
│   │   ├── block-navigation.js
│   │   ├── favicon.png
│   │   ├── index.html
│   │   ├── prettify.css
│   │   ├── prettify.js
│   │   ├── sort-arrow-sprite.png
│   │   └── sorter.js
│   ├── index.html
│   ├── openapi.json
│   ├── tarpaulin-report.html
│   └── use-case.png
├── frontend/
│   ├── dist/
│   │   ├── assets/
│   │   │   ├── index-BY7Lhp4b.css
│   │   │   └── index-C2Y95dqw.js
│   │   ├── 404_logo.png
│   │   ├── 404.png
│   │   ├── ai-pic.png
│   │   ├── christian.jpeg
│   │   ├── earth.png
│   │   ├── ellie.jpeg
│   │   ├── ethan.jpeg
│   │   ├── index.html
│   │   ├── left-arrow.png
│   │   ├── logo_background.jpg
│   │   ├── logo_rounded.png
│   │   ├── logo_white.png
│   │   ├── logo.png
│   │   ├── nick.jpeg
│   │   ├── peter.jpeg
│   │   ├── plane.jpg
│   │   ├── react.svg
│   │   ├── rightarrow.png
│   │   ├── robot-svgrepo-co...
│   │   ├── user-pfp-temp.png
│   │   └── vite.svg
│   ├── node_modules/
│   ├── public/
│   │   ├── 404_logo.png
│   │   ├── 404.png
│   │   ├── ai-pic.png
│   │   ├── christian.jpeg
│   │   ├── earth.png
│   │   ├── ellie.jpeg
│   │   ├── ethan.jpeg
│   │   ├── left-arrow.png
│   │   ├── logo_background.jpg
│   │   ├── logo_rounded.png
│   │   ├── logo_white.png
│   │   ├── logo.png
│   │   ├── nick.jpeg
│   │   ├── peter.jpeg
│   │   ├── plane.jpg
│   │   ├── react.svg
│   │   ├── rightarrow.png
│   │   ├── robot-svgrepo-co...
│   │   ├── user-pfp-temp.png
│   │   └── vite.svg
│   ├── src/
│   │   ├── api/
│   │   │   ├── account.ts
│   │   │   ├── home.ts
│   │   │   └── itinerary.ts
│   │   ├── assets/
│   │   ├── components/
│   │   │   ├── AboutSection.tsx
│   │   │   ├── AuthLayout.tsx
│   │   │   ├── ChatMessage.tsx
│   │   │   ├── ChatWindow.tsx
│   │   │   ├── CompactItinerary.tsx
│   │   │   ├── ContextWindow.tsx
│   │   │   ├── DoneSection.tsx
│   │   │   ├── EventCard.tsx
│   │   │   ├── Footer.tsx
│   │   │   ├── GlobalProvider.tsx
│   │   │   ├── HeroSection.tsx
│   │   │   ├── Itinerary.tsx
│   │   │   ├── ItinerarySideBar.tsx
│   │   │   ├── Loading.tsx
│   │   │   ├── MessageInput.tsx
│   │   │   ├── Navbar.tsx
│   │   │   ├── PrevChatSideBar.tsx
│   │   │   ├── ProtectedLink.tsx
│   │   │   ├── ProtectedRoute.tsx
│   │   │   ├── ReviewsSection.tsx
│   │   │   ├── TeamSection.tsx
│   │   │   ├── Toast.tsx
│   │   │   ├── UnassignedEvent.tsx
│   │   │   ├── UserMessageActions.tsx
│   │   │   ├── ValueSection.tsx
│   │   │   └── ViewPageSidebar.tsx
│   │   ├── helpers/
│   │   │   ├── account.ts
│   │   │   ├── config.ts
│   │   │   ├── global.ts
│   │   │   └── itinerary.ts
│   │   ├── models/
│   │   │   ├── account.ts
│   │   │   ├── chat.ts
│   │   │   ├── home.ts
│   │   │   └── itinerary.ts
│   │   ├── pages/
│   │   │   ├── Account.tsx
│   │   │   ├── Home.tsx
│   │   │   ├── Index.tsx
│   │   │   ├── Itineraries.tsx
│   │   │   ├── Login.tsx
│   │   │   ├── NotFound.tsx
│   │   │   ├── Preferences.tsx
│   │   │   ├── SignUp.tsx
│   │   │   └── ViewItinerary.tsx
│   │   ├── styles/
│   │   │   ├── Account.css
│   │   │   ├── App.css
│   │   │   ├── ChatMessage.css
│   │   │   ├── ChatWindow.css
│   │   │   ├── CompactItinerary.css
│   │   │   ├── ContextWindow.css
│   │   │   ├── EventCard.css
│   │   │   ├── FinishAccountPo.css
│   │   │   ├── Footer.css
│   │   │   ├── Home.css
│   │   │   ├── Index.css
│   │   │   ├── Itinerary.css
│   │   │   ├── ItinerarySideBar.css
│   │   │   ├── Login.css
│   │   │   ├── Navbar.css
│   │   │   ├── NotFound.css
│   │   │   ├── PrevChatSideBar.css
│   │   │   ├── SignUp.css
│   │   │   ├── Toast.css
│   │   │   ├── UserMessageActions.css
│   │   │   └── ViewPageSidebar.css
│   │   ├── App.tsx
│   │   ├── main.tsx
│   │   └── vite-env.d.ts
│   ├── tests/
│   │   ├── customFetch.ts
│   │   ├── genTestApi.js
│   │   ├── integration.test.ts
│   │   ├── testApi.ts
│   │   └── unit.test.ts
│   ├── .env
│   ├── .prettierignore
│   ├── .prettierrc
│   ├── eslint.config.js
│   ├── index.html
│   ├── package-lock.json
│   ├── package.json
│   ├── README.md
│   ├── tsconfig.app.json
│   ├── tsconfig.json
│   ├── tsconfig.node.json
│   ├── vite.config.ts
│   └── vitest.config.ts
├── logs/
│   ├── crash.log
│   └── latest.log
├── migrations/
│   └── 01_migration_script...
├── node_modules/
├── src/
│   ├── agent/
│   │   ├── configs/
│   │   │   ├── constraint.rs
│   │   │   ├── example.rs
│   │   │   ├── mod.rs
│   │   │   ├── optimizer.rs
│   │   │   ├── orchestrator.rs
│   │   │   └── research.rs
│   │   ├── models/
│   │   │   ├── context.rs
│   │   │   ├── event.rs
│   │   │   ├── itinerary.rs
│   │   │   ├── mod.rs
│   │   │   └── user.rs
│   │   ├── tools/
│   │   │   ├── constraint.rs
│   │   │   ├── example.rs
│   │   │   ├── mod.rs
│   │   │   ├── optimizer.rs
│   │   │   ├── orchestrator.rs
│   │   │   └── research.rs
│   │   └── mod.rs
│   ├── controllers/
│   │   ├── account.rs
│   │   ├── chat.rs
│   │   ├── itinerary.rs
│   │   └── mod.rs
│   ├── http_models/
│   │   ├── account.rs
│   │   ├── chat_session.rs
│   │   ├── event.rs
│   │   ├── itinerary.rs
│   │   ├── message.rs
│   │   ├── mod.rs
│   │   └── README.md
│   ├── models/
│   ├── sql_models/
│   │   ├── account.rs
│   │   ├── event_list.rs
│   │   ├── itinerary.rs
│   │   ├── message.rs
│   │   ├── mod.rs
│   │   └── README.md
│   ├── db.rs
│   ├── error.rs
│   ├── global.rs
│   ├── log.rs
│   ├── main.rs
│   ├── middleware.rs
│   ├── swagger.rs
│   └── tests.rs
├── target/
├── tests/
├── .env
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── CODE_OF_CONDUCT.md
├── CODEOWNERS
├── CONTRIBUTING.md
├── docker-compose.yml
├── PULL_REQUEST_TEMPLATE.md
├── README.md
├── rustfmt.toml
└── tarpaulin.toml
```

## Key Directories

### Backend (Rust)
- **src/** - Main backend source code
  - **agent/** - AI agent configurations, models, and tools
  - **controllers/** - API endpoint handlers (account, chat, itinerary)
  - **http_models/** - Request/response data structures
  - **sql_models/** - Database model definitions
  
### Frontend (React + TypeScript)
- **frontend/src/** - Frontend application source
  - **api/** - API client functions
  - **components/** - React components
  - **pages/** - Page-level components
  - **models/** - TypeScript type definitions
  - **helpers/** - Utility functions
  - **styles/** - CSS stylesheets

### Configuration & Documentation
- **.github/** - GitHub Actions workflows and issue templates
- **docs/** - Documentation and code coverage reports
- **migrations/** - Database migration scripts
- **logs/** - Application logs (crash.log, latest.log)