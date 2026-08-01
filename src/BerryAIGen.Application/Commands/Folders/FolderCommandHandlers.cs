using BerryAIGen.Domain.Interfaces.Repositories;
using BerryAIGen.Domain.ValueObjects;
using MediatR;

namespace BerryAIGen.Application.Commands.Folders;

public class CreateFolderCommandHandler : IRequestHandler<CreateFolderCommand, Guid>
{
    private readonly IFolderRepository _folderRepository;
    private readonly IUnitOfWork _unitOfWork;

    public CreateFolderCommandHandler(IFolderRepository folderRepository, IUnitOfWork unitOfWork)
    {
        _folderRepository = folderRepository;
        _unitOfWork = unitOfWork;
    }

    public async Task<Guid> Handle(CreateFolderCommand request, CancellationToken cancellationToken)
    {
        var parentFolderId = request.ParentFolderId.HasValue ? FolderId.From(request.ParentFolderId.Value) : null;
        var folder = Domain.Entities.Folder.Create(FilePath.From(request.Path), request.Recursive, parentFolderId);
        await _folderRepository.AddAsync(folder, cancellationToken);
        await _unitOfWork.SaveChangesAsync(cancellationToken);
        return folder.Id;
    }
}

public class UpdateFolderCommandHandler : IRequestHandler<UpdateFolderCommand, Unit>
{
    private readonly IFolderRepository _folderRepository;
    private readonly IUnitOfWork _unitOfWork;

    public UpdateFolderCommandHandler(IFolderRepository folderRepository, IUnitOfWork unitOfWork)
    {
        _folderRepository = folderRepository;
        _unitOfWork = unitOfWork;
    }

    public async Task<Unit> Handle(UpdateFolderCommand request, CancellationToken cancellationToken)
    {
        var folder = await _folderRepository.GetByIdAsync(FolderId.From(request.Id), cancellationToken)
            ?? throw new InvalidOperationException($"Folder {request.Id} not found");

        if (request.IsWatched) folder.StartWatching();
        else folder.StopWatching();

        _folderRepository.Update(folder);
        await _unitOfWork.SaveChangesAsync(cancellationToken);
        return Unit.Value;
    }
}

public class DeleteFolderCommandHandler : IRequestHandler<DeleteFolderCommand, Unit>
{
    private readonly IFolderRepository _folderRepository;
    private readonly IUnitOfWork _unitOfWork;

    public DeleteFolderCommandHandler(IFolderRepository folderRepository, IUnitOfWork unitOfWork)
    {
        _folderRepository = folderRepository;
        _unitOfWork = unitOfWork;
    }

    public async Task<Unit> Handle(DeleteFolderCommand request, CancellationToken cancellationToken)
    {
        var folder = await _folderRepository.GetByIdAsync(FolderId.From(request.Id), cancellationToken)
            ?? throw new InvalidOperationException($"Folder {request.Id} not found");

        _folderRepository.Delete(folder);
        await _unitOfWork.SaveChangesAsync(cancellationToken);
        return Unit.Value;
    }
}