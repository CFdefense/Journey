# Contributing Guide
We use GitHub Projects to allocate and track work.
## Issues
Issues will be used to track epics and stories. When creating an issue, choose from one of the provided templates.
The issue will be added to the project board. Make sure to fill in as much info as possible (priority, time estimate, etc...).
If an issue blocks another, make sure to note that in the issue settings.
### Epics
An epic is a large feature or change that needs to be added to the project.

An epic should have a high-level description, and a low-level acceptance criteria defining the business requirements.
An epic may have developer notes for very specific implementation details.

Epic stages:
* Backlog - Epics start in backlog where a high-level feature request is described.
* Ready - The epic requirements are defined and being broken down into stories.
* In Progress - Stories within the epic are being worked on.
* In Review - All stories within the epic have been accepted.
* Accepted - All requirements have been implemented.
### Stories
An story is a small change which works towards completing an epic.

An story should have a high-level description, and a low-level acceptance criteria defining the scope of the story.
The story must also link to a parent epic, which will automatically update the list of stories in that epic.
An story may have developer notes for very specific implementation details.

* Backlog - Stories start in backlog where a small change to the project described.
* Ready - The story is ready to be picked up.
* In Progress - The story is actively being worked on.
* In Review - The story is claimed to be completed according to its acceptance criteria, and other team members must approve it or request changes.
* Accepted - The story is implemented and complete according to approving team members.
### Miscellaneous
A miscellaneous issue which defines a small change, but does not belong to an epic. Misc should be treated as a story without an epic.
## Pull Requests
Pull requests and stories/miscellaneous issues have a one-to-one relationship. PRs must be approved by another team member to be merged.
Try to avoid making PRs until all changes are implemented, or use a draft-PR if you're unsure. Make sure to link PRs to issues when applicable.
## Branches
When doing any work, create a separate branch with a name that corresponds to the story/misc issue. When a story has been completed, avoid deleting its branch until after the epic is done and accepted.