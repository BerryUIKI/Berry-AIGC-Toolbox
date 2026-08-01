using MediatR;

namespace BerryAIGen.Application.Queries.Folders;

public record GetFolderByIdQuery : IRequest<Application.DTOs.FolderDto?>
{
    public Guid Id { get; init; }
}

public record GetFoldersQuery : IRequest<List<Application.DTOs.FolderSummaryDto>>
{
    public bool? WatchedOnly { get; init; }
}

public record GetSubfoldersQuery : IRequest<List<Application.DTOs.FolderSummaryDto>>
{
    public Guid ParentFolderId { get; init; }
}