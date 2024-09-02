import type { Probot } from 'probot';
import { type CodeOwnersEntry, parseCodeOwners } from './codeowners';

type When =
  | 'isAnyFilePathMatch'
  | 'isPRBodyMatch'
  | 'isPRTitleMatch'
  | 'isPRAuthorMatch'
  | 'isPRAuthorCompanyMatch'
  | 'isAnyFileOwnedByMatch';

type RemoveIsPrefix<T extends string> = T extends `is${infer R}` ? R : T;

export default (app: Probot) => {
  app.on(
    ['pull_request.opened', 'pull_request.synchronize'],
    async (context) => {
      const repo = context.repo();
      const labels = (
        await context.octokit.issues.listLabelsForRepo(repo)
      ).data.map((label) => label.name);

      const { config } = await context.octokit.config.get<{
        labeler: {
          settings?: {
            codeOwnersPath?: string;
          };
          labels: {
            label: string;
            condition: string;
            when: Record<
              When | `isNot${Capitalize<RemoveIsPrefix<When>>}`,
              string | undefined
            >;
          }[];
        };
      }>({
        ...repo,
        path: '.github/orchestrator.yml',
      });

      const codeOwnersPath =
        config.labeler.settings?.codeOwnersPath ?? '.github/CODEOWNERS';
      const codeOwnersFile = await context.octokit.rest.repos
        .getContent({
          ...repo,
          path: config.labeler.settings?.codeOwnersPath ?? '.github/CODEOWNERS',
        })
        .catch(() => undefined);

      let codeOwners: CodeOwnersEntry[] | undefined;
      const labelsToAdd: typeof labels = [];

      for await (const { label, when } of config.labeler.labels) {
        const conditions = await Promise.all(
          Object.entries(when).map(async ([when, value]) => {
            if (value == null) return;

            const isNot = when.startsWith('isNot');
            const condition = (isNot ? when.slice(5) : when) as When;

            const getCondition = async (): Promise<boolean> => {
              switch (condition) {
                case 'isPRTitleMatch': {
                  const regex = new RegExp(value);
                  return regex.test(context.payload.pull_request.title);
                }
                case 'isPRBodyMatch': {
                  const regex = new RegExp(value);
                  return Boolean(
                    context.payload.pull_request.body &&
                      regex.test(context.payload.pull_request.body),
                  );
                }
                case 'isPRAuthorMatch': {
                  const regex = new RegExp(value, 'i');
                  return (
                    context.payload.action === 'opened' &&
                    regex.test(context.payload.sender.login)
                  );
                }
                case 'isPRAuthorCompanyMatch': {
                  const regex = new RegExp(value, 'i');

                  return (
                    await context.octokit.orgs.listForUser({
                      username: context.payload.sender.login,
                    })
                  ).data
                    .map((org) => org.login)
                    .some((org) => regex.test(org));
                }
                case 'isAnyFileOwnedByMatch': {
                  if (
                    codeOwnersFile == null ||
                    !('type' in codeOwnersFile.data) ||
                    codeOwnersFile.data.type !== 'file'
                  ) {
                    app.log.error(
                      `CODEOWNERS file not found at ${codeOwnersPath}`,
                    );
                    return false;
                  }

                  codeOwners ??= parseCodeOwners(codeOwnersFile.data.content);

                  const files = (
                    await context.octokit.pulls.listFiles({
                      ...repo,
                      pull_number: context.payload.pull_request.number,
                    })
                  ).data;

                  return files.some((file) => {
                    const owners = codeOwners!.find(({ pattern }) =>
                      new RegExp(pattern).test(file.filename),
                    )?.owners;

                    return owners
                      ? owners.includes(context.payload.sender.login)
                      : false;
                  });
                }
                case 'isAnyFilePathMatch': {
                  const files = (
                    await context.octokit.pulls.listFiles({
                      ...repo,
                      pull_number: context.payload.pull_request.number,
                    })
                  ).data;

                  const regex = new RegExp(value);
                  return files.some((file) => regex.test(file.filename));
                }
              }
            };

            return !isNot && (await getCondition());
          }),
        );

        if (conditions.every(Boolean)) {
          labelsToAdd.push(label);
        }
      }

      if (labelsToAdd.length > 0)
        context.octokit.issues.addLabels(
          context.issue({
            issue_number: context.payload.pull_request.number,
            labels: labelsToAdd,
          }),
        );
    },
  );
};
