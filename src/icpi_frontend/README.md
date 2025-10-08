# ICPI Transfer Portal

A React-based frontend application for transferring ICP, ckUSDT, and ICPI tokens on the Internet Computer.

## Features

- **Internet Identity Authentication**: Secure login using Internet Identity
- **Multi-Token Support**: Transfer ICP, ckUSDT, and ICPI tokens
- **Real-time Balance Display**: View current token balances
- **Principal ID Validation**: Ensures valid recipient addresses
- **Transaction Status Feedback**: Clear success/error messages

## Tech Stack

- React 18 with TypeScript
- Redux Toolkit for state management
- @dfinity/agent for IC interactions
- ic-use-actor for actor management
- ic-use-internet-identity for authentication
- Vite for build tooling

## Project Structure

```
src/
├── actors/           # Actor provider components
├── components/       # Reusable UI components
├── contexts/         # React contexts (Auth, Actors)
├── features/         # Feature-specific code
│   └── transfers/    # Transfer functionality
├── hooks/           # Custom React hooks
├── providers/       # Context providers
├── store/           # Redux store configuration
├── styles/          # CSS styles
└── utils/           # Utility functions
```

## Setup

1. Install dependencies:
```bash
npm install
```

2. Generate canister declarations:
```bash
dfx generate
```

3. Start the development server:
```bash
npm start
```

## Usage

1. **Connect Wallet**: Click "Connect Wallet" to authenticate with Internet Identity
2. **Select Token**: Choose between ICP, ckUSDT, or ICPI
3. **Enter Amount**: Specify the amount to transfer
4. **Enter Recipient**: Provide the recipient's Principal ID
5. **Send**: Click send to execute the transfer

## Token Information

- **ICP**: Native Internet Computer token (8 decimals)
- **ckUSDT**: Chain-key USDT (6 decimals)
- **ICPI**: IC Portfolio Index token (8 decimals)

## Development

### Adding New Tokens

To add support for additional ICRC-1 tokens:

1. Create actor context in `src/contexts/actors/`
2. Create actor provider in `src/actors/`
3. Create hook in `src/hooks/actors/`
4. Add to TOKEN_OPTIONS in TransferPanel.tsx

### Building for Production

```bash
npm run build
```

## Security Notes

- Always validate Principal IDs before sending
- Session expiry is handled automatically
- Fees are calculated and displayed before transactions
- Private keys never leave the Internet Identity