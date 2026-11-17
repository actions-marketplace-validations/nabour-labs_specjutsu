# billing Specification

## Purpose
Charge customers monthly and send them an invoice they can download as a PDF.

## Requirements

### Requirement: Monthly invoice
The system SHALL create an invoice on the first day of each month.

#### Scenario: BILL-001 An invoice is created
- **WHEN** the month starts
- **THEN** each active customer has a new invoice

### Requirement: Invoice download
The system SHALL serve each invoice as a PDF.

#### Scenario: BILL-002 An invoice is downloaded
- **WHEN** the customer opens an invoice
- **THEN** a PDF is returned
