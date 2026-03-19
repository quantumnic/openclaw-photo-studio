# Governance — OpenClaw Photo Studio

## Model: Benevolent Steward + Technical Council

### Project Steward (1 person)
- Final authority on licensing and trademark decisions
- Can veto license changes
- Can be overridden by TC on purely technical questions (4/5 supermajority)
- Can transfer the role to a TC-confirmed successor
- Initial steward: Project founder

### Technical Council (TC, 3-7 members)
- Elected by active contributors (≥5 merged PRs in past 12 months)
- 2-year terms, staggered (half the council every year)
- Decides on: Architecture RFCs, breaking changes, release readiness
- At least 1 seat reserved for "community-at-large" (non-core contributor)
- Simple majority for decisions, supermajority (4/5) to override steward on technical matters
- Meetings: Monthly, public minutes

### Module Maintainers
- Responsible for specific modules (RAW Engine, UI, Catalog, Plugin System, etc.)
- Nominated by TC, confirmed by community vote
- Required to review PRs in their module within 5 business days
- Can appoint Junior Maintainers

### Contributors
- Anyone with signed CLA and ≥1 merged PR
- Voting rights for TC elections after 5 merged PRs
- Can propose RFCs

## Decision Process

| Decision Type | Who Decides | Process |
|--------------|-------------|---------|
| Bug fix | Module Maintainer | PR review + merge |
| Small feature | Module Maintainer | PR review + merge |
| Large feature | TC | RFC → 14-day review → TC vote |
| Architecture change | TC | RFC → 14-day review → TC vote |
| Breaking change | TC | RFC → 14-day review → TC vote |
| License change | Steward + TC | RFC → 30-day review → TC supermajority + Steward approval |
| Trademark use | Steward | Case-by-case |
| Release | TC | Release checklist + TC approval |
| New TC member | Contributors | Nomination + election |
| New Maintainer | TC | Nomination + TC vote |

## Conflict Resolution

1. Discussion in issue/PR
2. Module maintainer decision
3. TC appeal (if disagreement persists)
4. Steward final decision (only for non-technical matters)

## Transparency

- All TC decisions are documented in `governance/decisions/`
- All RFC votes are public
- Financial matters (if applicable) are reported quarterly
- Meeting minutes are published within 7 days
