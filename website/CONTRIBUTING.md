# Contributing to Docs

Thank you for your interest in contributing to the documentation! Here are some guidelines to help you get started.

## Pre-requisites

Before contributing, please ensure you have the following set up:

1. Make sure you are in the `website` directory of the project. (`cd website`)
2. We are using a specific version of Node, just to make sure everything works as expected. You can use [nvm](https://github.com/nvm-sh/nvm) to use the correct version:
   ```bash
   nvm use
   ```
3. We are using [pnpm](https://pnpm.io/) as our package manager. You can install it, and use the expected version, via `corepack`:
   ```bash
   corepack enable
   ```
4. Now you are ready to install the project dependencies:
   ```bash
   pnpm install
   ```

## Making Changes

Once you have the pre-requisites set up, you can run the app:

```bash
pnpm start
```

> [!NOTE]
> If you're using VSCode, you should add the website directory to your workspace. File -> Add Folder to Workspace... - Select the `website` directory. This will make TypeScript work correctly, since it will read the `tsconfig.json` file from the `website` directory, as if it were the root of the project.

The landing page is located at [`src/pages/index.tsx`](./src/pages/index.tsx). You can edit this file to make changes to the landing page.

And to change the docs, you can edit the files in the [`docs`](../docs) directory. The documentation is written in Markdown, so you can use standard Markdown syntax to format your content, or you can use MDX to include React components in your documentation.