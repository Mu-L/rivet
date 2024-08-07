/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { DisplayName as common$$displayName } from "../../../../common/types/DisplayName";
import { ExternalLinks as group_common$$externalLinks } from "./ExternalLinks";
import { common, group } from "../../../../index";

export const Handle: core.serialization.ObjectSchema<serializers.group.Handle.Raw, Rivet.group.Handle> =
    core.serialization.object({
        groupId: core.serialization.property("group_id", core.serialization.string()),
        displayName: core.serialization.property("display_name", common$$displayName),
        avatarUrl: core.serialization.property("avatar_url", core.serialization.string().optional()),
        external: group_common$$externalLinks,
        isDeveloper: core.serialization.property("is_developer", core.serialization.boolean().optional()),
    });

export declare namespace Handle {
    interface Raw {
        group_id: string;
        display_name: common.DisplayName.Raw;
        avatar_url?: string | null;
        external: group.ExternalLinks.Raw;
        is_developer?: boolean | null;
    }
}
