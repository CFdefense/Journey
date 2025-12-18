// Agent progress configuration mapping
// Maps progress status strings to human-readable messages and profile pictures

export enum AgentProgress {
	Ready = "Ready",
	// Task Agent Tools
	RetrieveUserProfile = "RetrieveUserProfile",
	RetrieveChatContext = "RetrieveChatContext",
	UpdateTripContext = "UpdateTripContext",
	UpdateChatTitle = "UpdateChatTitle",
	AskForClarification = "AskForClarification",
	// Research Agent
	Searching = "Searching",
	Geocoding = "Geocoding",
	SearchingEvents = "SearchingEvents",
	// Constraint Agent
	Filtering = "Filtering",
	CheckingConstraints = "CheckingConstraints",
	// Task/Scheduling Agent
	Scheduling = "Scheduling",
	// Optimizer Agent
	Optimizing = "Optimizing",
	RankingEvents = "RankingEvents",
	// Final Response
	FinalizingItinerary = "FinalizingItinerary"
}

export interface AgentInfo {
	name: string;
	profilePic: string;
	message: string;
}

export const AGENT_PROGRESS_MAP: Record<AgentProgress, AgentInfo> = {
	[AgentProgress.Ready]: {
		name: "System",
		profilePic: "/logo.png",
		message: "Ready to help with your travel plans"
	},
	// Task Agent Tools
	[AgentProgress.RetrieveUserProfile]: {
		name: "Task Agent",
		profilePic: "/task-white.png",
		message: "Loading your profile and preferences"
	},
	[AgentProgress.RetrieveChatContext]: {
		name: "Task Agent",
		profilePic: "/task-white.png",
		message: "Loading chat history"
	},
	[AgentProgress.UpdateTripContext]: {
		name: "Task Agent",
		profilePic: "/task-white.png",
		message: "Analyzing your trip details"
	},
	[AgentProgress.UpdateChatTitle]: {
		name: "Task Agent",
		profilePic: "/task-white.png",
		message: "Organizing your chat"
	},
	[AgentProgress.AskForClarification]: {
		name: "Task Agent",
		profilePic: "/task-white.png",
		message: "Preparing clarification questions"
	},
	// Research Agent
	[AgentProgress.Searching]: {
		name: "Researcher",
		profilePic: "/researcher-white.png",
		message: "Searching for destinations"
	},
	[AgentProgress.Geocoding]: {
		name: "Researcher",
		profilePic: "/researcher-white.png",
		message: "Finding location coordinates"
	},
	[AgentProgress.SearchingEvents]: {
		name: "Researcher",
		profilePic: "/researcher-white.png",
		message: "Discovering nearby activities and places"
	},
	// Constraint Agent
	[AgentProgress.Filtering]: {
		name: "Constraint Agent",
		profilePic: "/constraint-white.png",
		message: "Filtering options based on your preferences"
	},
	[AgentProgress.CheckingConstraints]: {
		name: "Constraint Agent",
		profilePic: "/constraint-white.png",
		message: "Checking accessibility and dietary requirements"
	},
	// Scheduling
	[AgentProgress.Scheduling]: {
		name: "Orchestrator",
		profilePic: "/orchestrator-white-crop.png",
		message: "Drafting your itinerary"
	},
	// Optimizer
	[AgentProgress.Optimizing]: {
		name: "Optimizer",
		profilePic: "/optimizer-white.png",
		message: "Optimizing your itinerary"
	},
	[AgentProgress.RankingEvents]: {
		name: "Optimizer",
		profilePic: "/optimizer-white.png",
		message: "Ranking events based on your preferences"
	},
	// Final Response
	[AgentProgress.FinalizingItinerary]: {
		name: "Orchestrator",
		profilePic: "/orchestrator-white-crop.png",
		message: "Finalizing your itinerary"
	}
};
