package planner

// Bead Template Schema - Validates generated bead structure
// This ensures generated beads have all required sections

#Bead: {
	// Metadata
	id:              string & =~"^intent-cli-[0-9]{14}-[a-z0-9]{8}$"
	title:           string & =~"^[a-z-]+: .+$"
	type:            "feature" | "bug" | "task" | "epic" | "chore"
	priority:        >=0 & <=4
	effort_estimate: "15min" | "30min" | "1hr" | "2hr" | "4hr"
	labels: [...string]

	// Section 0: Clarifications
	clarifications: {
		clarification_status: "RESOLVED" | "HAS_OPEN_QUESTIONS"
		resolved_clarifications?: [...{
			question:   string
			answer:     string
			decided_by: string
			date:       string
		}]
		open_clarifications?: [...{
			question:              string
			context:               string
			options: [...string]
			default_if_unresolved: string
		}]
		assumptions?: [...{
			assumption:         string
			validation_method:  string
			risk_if_wrong:      string
		}]
	}

	// Section 1: EARS Requirements
	ears_requirements: {
		ubiquitous: [...string] & list.MinItems(1)
		event_driven: [...{
			trigger: string
			shall:   string
		}] & list.MinItems(1)
		state_driven?: [...{
			state: string
			shall: string
		}]
		optional?: [...{
			condition: string
			shall:     string
		}]
		unwanted: [...{
			condition:  string
			shall_not:  string
			because:    string
		}] & list.MinItems(1)
		complex?: [...{
			state:   string
			trigger: string
			shall:   string
		}]
	}

	// Section 2: KIRK Contracts
	contracts: {
		preconditions: {
			auth_required: bool
			required_inputs?: [...{
				field:       string
				type:        string
				constraints: string
			}]
			system_state: [...string] & list.MinItems(1)
		}
		postconditions: {
			state_changes: [...string]
			return_guarantees?: [...{
				field:     string
				guarantee: string
			}]
			side_effects?: [...string]
		}
		invariants: [...string] & list.MinItems(1)
	}

	// Section 2.5: Research Requirements
	research_requirements: {
		files_to_read?: [...{
			path:            string
			what_to_extract: string
			document_in:     string
		}]
		patterns_to_find?: [...{
			pattern:            string
			purpose:            string
			expected_locations: string
		}]
		prior_art?: [...{
			feature:       string
			location:      string
			what_to_learn: string
		}]
		external_docs?: [...{
			url:     string
			section: string
			extract: string
		}]
		research_questions?: [...{
			question: string
			answered: bool
			answer?:  string
		}]
		research_complete_when: [...string] & list.MinItems(1)
	}

	// Section 3: Inversions
	inversions: {
		security_failures?: [...{
			failure:     string
			prevention:  string
			test_for_it: string
		}]
		usability_failures?: [...{
			failure:     string
			prevention:  string
			test_for_it: string
		}]
		data_integrity_failures?: [...{
			failure:     string
			prevention:  string
			test_for_it: string
		}]
		integration_failures?: [...{
			failure:     string
			prevention:  string
			test_for_it: string
		}]
	}

	// Section 4: ATDD Tests
	acceptance_tests: {
		happy_paths: [...{
			name:            string
			given:           string
			when:            string
			then: [...string]
			real_input:      string
			expected_output: string
		}] & list.MinItems(1)

		error_paths: [...{
			name:            string
			given:           string
			when:            string
			then: [...string]
			real_input:      string
			expected_output?: string
			expected_error:  string
		}] & list.MinItems(1)

		edge_cases?: [...{
			name:     string
			scenario: string
			input:    string
			expected: string
		}]

		contract_tests?: [...{
			name:     string
			verifies: string
			test:     string
		}]
	}

	// Section 5: E2E Tests
	e2e_tests: {
		pipeline_test: {
			name:        string
			description: string
			setup?: {
				files_to_create?: [...{
					path:    string
					content: string
				}]
				environment?: [...string]
				precondition_commands?: [...string]
			}
			execute: {
				command:    string
				stdin?:     string
				timeout_ms: number
			}
			verify: {
				exit_code: number
				stdout_contains?: [...string]
				stdout_matches_json?: {...}
				files_created?: [...string]
				files_not_modified?: [...string]
				side_effects?: [...string]
			}
			cleanup?: {
				commands?: [...string]
				files_to_delete?: [...string]
			}
		}
		e2e_scenarios?: [...{
			name:        string
			description: string
			steps: [...{
				action: string
				verify: string
			}]
		}]
	}

	// Section 5.5: Verification Checkpoints
	verification_checkpoints: {
		gate_0_research: {
			name:              string
			must_pass_before:  string
			checks: [...string]
			evidence_required: [...string]
		}
		gate_1_tests: {
			name:              string
			must_pass_before:  string
			checks: [...string]
			evidence_required: [...string]
		}
		gate_2_implementation: {
			name:              string
			must_pass_before:  string
			checks: [...string]
			evidence_required: [...string]
		}
		gate_3_integration: {
			name:              string
			must_pass_before:  string
			checks: [...string]
			evidence_required: [...string]
		}
	}

	// Section 6: Implementation Tasks
	implementation_tasks: {
		phase_0_research?: {
			parallelizable: bool
			tasks: [...{
				task:           string
				done_when:      string
				parallel_group: string
			}]
		}
		phase_1_tests_first: {
			parallelizable:  bool
			gate_required?:  string
			tasks: [...{
				task:           string
				done_when:      string
				parallel_group: string
			}]
		}
		phase_2_implementation: {
			parallelizable: bool
			gate_required?: string
			tasks: [...{
				task:       string
				done_when:  string
				depends_on?: string
			}]
		}
		phase_3_integration?: {
			parallelizable: bool
			gate_required?: string
			tasks: [...{
				task:      string
				done_when: string
			}]
		}
		phase_4_verification: {
			parallelizable: bool
			gate_required?: string
			tasks: [...{
				task:           string
				done_when:      string
				parallel_group: string
			}]
		}
	}

	// Section 7: Failure Modes
	failure_modes: {
		failure_modes: [...{
			symptom:      string
			likely_cause: string
			where_to_look: [...{
				file:          string
				what_to_check: string
			}]
			fix_pattern: string
		}]
	}

	// Section 7.5: Anti-Hallucination
	anti_hallucination: {
		read_before_write: [...{
			file:                       string
			must_read_first:            bool
			key_sections_to_understand: [...string]
		}]
		apis_that_exist?: [...string]
		apis_that_do_not_exist?: [...string]
		no_placeholder_values: [...string]
		git_verification?: {
			before_claiming_done: string
		}
	}

	// Section 7.6: Context Survival
	context_survival: {
		progress_file: {
			path:   string
			format: string
		}
		tests_status_file?: {
			path:              string
			update_frequency?: string
		}
		research_notes_file?: {
			path:     string
			contains: [...string]
		}
		git_checkpoints?: {
			frequency:      string
			message_format: string
			purpose:        string
		}
		recovery_instructions: string
	}

	// Section 8: Completion Checklist
	completion_checklist: {
		tests: [...string] & list.MinItems(4)
		code: [...string] & list.MinItems(2)
		ci: [...string] & list.MinItems(1)
		documentation?: [...string]
	}

	// Section 9: Context
	context: {
		related_files?: [...{
			path:      string
			relevance: string
		}]
		similar_implementations?: [...string]
		external_references?: [...string]
		codebase_patterns?: [...{
			pattern:           string
			example_location:  string
			how_to_apply:      string
		}]
	}

	// Section 10: AI Hints
	ai_hints: {
		do: [...string] & list.MinItems(3)
		do_not: [...string] & list.MinItems(3)
		language_guidance?: {
			avoid: [...string]
			use_instead: [...string]
		}
		action_guidance?: string
		parallel_execution?: string
		incremental_progress?: string
		code_patterns?: [...{
			name:      string
			use_when:  string
			example:   string
		}]
		constitution: [...string] & list.MinItems(2)
	}
}
