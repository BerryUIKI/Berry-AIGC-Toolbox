using MediatR;

namespace BerryAIGen.Application.Commands.Folders;

public record CreateFolderCommand : IRequest<Guid>
{
    public string Path { get; init; } = string.Empty;
    public bool Recursive { get; init; } = true;
    public Guid? ParentFolderId { get; init; }
}

public record UpdateFolderCommand : IRequest<Unit>
{
    public Guid Id { get; init; }
    public bool IsWatched { get; init; }
}

public record DeleteFolderCommand : IRequest<Unit>
{
    public Guid Id { get; init; }
}