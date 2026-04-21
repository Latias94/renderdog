import renderdoc as rd

from renderdog_qrenderdoc import is_drawcall_like


def normalize(value: str, case_sensitive: bool) -> str:
    if value is None:
        return ""
    if case_sensitive:
        return str(value)
    return str(value).lower()


def marker_path_join(marker_path) -> str:
    if not marker_path:
        return ""
    return "/".join([str(x) for x in marker_path])


class ActionFilter:
    def __init__(
        self,
        only_drawcalls: bool = False,
        marker_prefix: str = "",
        event_min=None,
        event_max=None,
        name_contains: str = "",
        marker_contains: str = "",
        case_sensitive: bool = False,
    ):
        self.only_drawcalls = bool(only_drawcalls)
        self.marker_prefix = str(marker_prefix or "")
        self.event_min = event_min
        self.event_max = event_max
        self.case_sensitive = bool(case_sensitive)
        self.name_contains = normalize(name_contains, self.case_sensitive)
        self.marker_contains = normalize(marker_contains, self.case_sensitive)

    def matches(self, action) -> bool:
        if self.marker_prefix:
            if not (
                action.marker_path_joined == self.marker_prefix
                or action.marker_path_joined.startswith(self.marker_prefix + "/")
            ):
                return False

        if self.only_drawcalls and not is_drawcall_like(action.flags):
            return False

        if self.event_min is not None and action.event_id < int(self.event_min):
            return False
        if self.event_max is not None and action.event_id > int(self.event_max):
            return False

        if self.name_contains:
            if self.name_contains not in normalize(action.name, self.case_sensitive):
                return False

        if self.marker_contains:
            if self.marker_contains not in normalize(
                action.marker_path_joined, self.case_sensitive
            ):
                return False

        return True


class ActionRecord:
    def __init__(self, action, parent_event_id, depth, name: str, flags: int, marker_path):
        self.action = action
        self.event_id = int(action.eventId)
        self.parent_event_id = (
            int(parent_event_id) if parent_event_id is not None else None
        )
        self.depth = int(depth)
        self.name = name
        self.flags = int(flags)
        self.marker_path = marker_path
        self.marker_path_joined = marker_path_join(marker_path)
        self.num_children = int(len(action.children))
        self.is_push_marker = bool(flags & rd.ActionFlags.PushMarker)

    @classmethod
    def from_action(
        cls,
        structured_file,
        action,
        marker_stack,
        parent_event_id,
        depth,
    ):
        name = str(action.GetName(structured_file))
        flags = int(action.flags)

        marker_path = list(marker_stack)
        if flags & rd.ActionFlags.PushMarker:
            marker_path.append(name)

        return cls(action, parent_event_id, depth, name, flags, marker_path)


def walk_actions(
    structured_file,
    actions,
    action_filter: ActionFilter,
    on_match,
    marker_stack=None,
    parent_event_id=None,
    depth: int = 0,
):
    if marker_stack is None:
        marker_stack = []

    for action in actions:
        record = ActionRecord.from_action(
            structured_file,
            action,
            marker_stack,
            parent_event_id,
            depth,
        )

        if action_filter.matches(record):
            on_match(record)

        next_marker_stack = record.marker_path if record.is_push_marker else marker_stack
        walk_actions(
            structured_file,
            action.children,
            action_filter,
            on_match,
            next_marker_stack,
            record.event_id,
            depth + 1,
        )
