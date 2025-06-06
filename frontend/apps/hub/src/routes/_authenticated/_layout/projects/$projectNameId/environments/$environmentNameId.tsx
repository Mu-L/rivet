import { ErrorComponent } from "@/components/error-component";
import {
	EnvironmentContextProvider,
	useEnvironment,
} from "@/domains/project/data/environment-context";
import { useProject } from "@/domains/project/data/project-context";
import * as Layout from "@/domains/project/layouts/project-layout";
import { useDialog } from "@/hooks/use-dialog";
import { usePageLayout } from "@/lib/compute-page-layout";
import { guardUuids } from "@/lib/guards";
import {
	type ErrorComponentProps,
	Outlet,
	createFileRoute,
} from "@tanstack/react-router";
import { zodValidator } from "@tanstack/zod-adapter";
import { z } from "zod";

function Modals() {
	const navigate = Route.useNavigate();
	const { gameId: projectId, nameId: projectNameId } = useProject();
	const { namespaceId: environmentId, nameId: environmentNameId } =
		useEnvironment();
	const { modal, buildId } = Route.useSearch();

	const ConfirmOuterbaseConnectionDialog =
		useDialog.ConfirmOuterbaseConnection.Dialog;

	const EditBuildTagsDialog = useDialog.EditBuildTags.Dialog;

	const handleOpenChange = (value: boolean) => {
		if (!value) {
			navigate({
				to: ".",
				search: (old) => ({ ...old, modal: undefined }),
			});
		}
	};

	return (
		<>
			<ConfirmOuterbaseConnectionDialog
				environmentId={environmentId}
				projectId={projectId}
				dialogProps={{
					open: modal === "database",
					onOpenChange: handleOpenChange,
				}}
			/>
			<EditBuildTagsDialog
				// biome-ignore lint/style/noNonNullAssertion: at this point we know buildId is defined
				buildId={buildId!}
				environmentNameId={environmentNameId}
				projectNameId={projectNameId}
				dialogProps={{
					open: modal === "edit-tags",
					onOpenChange: handleOpenChange,
				}}
			/>
		</>
	);
}

function EnvironmentErrorComponent(props: ErrorComponentProps) {
	const layout = usePageLayout();

	if (layout === "v2") {
		return (
			<div className="p-4">
				<div className="max-w-5xl mx-auto">
					<ErrorComponent {...props} />
				</div>
			</div>
		);
	}
	return <ErrorComponent {...props} />;
}

function environmentIdRoute() {
	const { environmentNameId } = Route.useParams();
	return (
		<EnvironmentContextProvider environmentNameId={environmentNameId}>
			<Outlet />
			<Modals />
		</EnvironmentContextProvider>
	);
}
const searchSchema = z.object({
	modal: z
		.enum(["database", "edit-tags", "create-actor", "go-to-actor"])
		.or(z.string())
		.optional(),
	buildId: z.string().optional().catch(undefined),
});
export const Route = createFileRoute(
	"/_authenticated/_layout/projects/$projectNameId/environments/$environmentNameId",
)({
	validateSearch: zodValidator(searchSchema),
	beforeLoad: async ({
		matches,
		location,
		params: { projectNameId, environmentNameId },
		context: { queryClient },
	}) => {
		await guardUuids({
			queryClient,
			location,
			projectNameId,
			environmentNameId,
		});
	},
	component: environmentIdRoute,
	errorComponent: EnvironmentErrorComponent,
	pendingComponent: Layout.Root.Skeleton,
});
