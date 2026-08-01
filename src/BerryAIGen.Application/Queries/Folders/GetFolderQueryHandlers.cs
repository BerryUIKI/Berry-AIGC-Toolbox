using BerryAIGen.Application.DTOs;
using BerryAIGen.Domain.Interfaces.Repositories;
using BerryAIGen.Domain.ValueObjects;
using MediatR;

namespace BerryAIGen.Application.Queries.Folders;

public class GetFolderByIdQueryHandler : IRequestHandler<GetFolderByIdQuery, FolderDto?>
{
    private readonly IFolderRepository _folderRepository;

    public GetFolderByIdQueryHandler(IFolderRepository folderRepository) => _folderRepository = folderRepository;

    public async Task<FolderDto?> Handle(GetFolderByIdQuery request, CancellationToken cancellationToken)
    {
        var folder = await _folderRepository.GetByIdAsync(FolderId.From(request.Id), cancellationToken);
        return folder == null ? null : new FolderDto
        {
            Id = folder.Id, Path = folder.Path, ParentFolderId = folder.ParentFolderId?.Value,
            IsWatched = folder.IsWatched, Recursive = folder.Recursive,
            ImageCount = folder.ImageCount, CreatedAt = folder.CreatedAt
        };
    }
}

public class GetFoldersQueryHandler : IRequestHandler<GetFoldersQuery, List<FolderSummaryDto>>
{
    private readonly IFolderRepository _folderRepository;

    public GetFoldersQueryHandler(IFolderRepository folderRepository) => _folderRepository = folderRepository;

    public async Task<List<FolderSummaryDto>> Handle(GetFoldersQuery request, CancellationToken cancellationToken)
    {
        var folders = request.WatchedOnly == true
            ? await _folderRepository.GetWatchedFoldersAsync(cancellationToken)
            : await _folderRepository.GetAllAsync(cancellationToken);

        return folders.Select(f => new FolderSummaryDto
        {
            Id = f.Id, Path = f.Path, IsWatched = f.IsWatched, ImageCount = f.ImageCount
        }).ToList();
    }
}

public class GetSubfoldersQueryHandler : IRequestHandler<GetSubfoldersQuery, List<FolderSummaryDto>>
{
    private readonly IFolderRepository _folderRepository;

    public GetSubfoldersQueryHandler(IFolderRepository folderRepository) => _folderRepository = folderRepository;

    public async Task<List<FolderSummaryDto>> Handle(GetSubfoldersQuery request, CancellationToken cancellationToken)
    {
        var folders = await _folderRepository.GetSubfoldersAsync(FolderId.From(request.ParentFolderId), cancellationToken);
        return folders.Select(f => new FolderSummaryDto
        {
            Id = f.Id, Path = f.Path, IsWatched = f.IsWatched, ImageCount = f.ImageCount
        }).ToList();
    }
}