using BerryAIGen.Domain.Interfaces.Repositories;
using BerryAIGen.Domain.ValueObjects;
using MediatR;

namespace BerryAIGen.Application.Commands.Albums;

/// <summary>
/// Handler for CreateAlbumCommand.
/// </summary>
public class CreateAlbumCommandHandler : IRequestHandler<CreateAlbumCommand, Guid>
{
    private readonly IAlbumRepository _albumRepository;
    private readonly IUnitOfWork _unitOfWork;

    public CreateAlbumCommandHandler(
        IAlbumRepository albumRepository,
        IUnitOfWork unitOfWork)
    {
        _albumRepository = albumRepository;
        _unitOfWork = unitOfWork;
    }

    public async Task<Guid> Handle(CreateAlbumCommand request, CancellationToken cancellationToken)
    {
        var album = Domain.Entities.Album.Create(request.Name, request.Description);
        await _albumRepository.AddAsync(album, cancellationToken);
        await _unitOfWork.SaveChangesAsync(cancellationToken);
        return album.Id;
    }
}

/// <summary>
/// Handler for UpdateAlbumCommand.
/// </summary>
public class UpdateAlbumCommandHandler : IRequestHandler<UpdateAlbumCommand, Unit>
{
    private readonly IAlbumRepository _albumRepository;
    private readonly IUnitOfWork _unitOfWork;

    public UpdateAlbumCommandHandler(
        IAlbumRepository albumRepository,
        IUnitOfWork unitOfWork)
    {
        _albumRepository = albumRepository;
        _unitOfWork = unitOfWork;
    }

    public async Task<Unit> Handle(UpdateAlbumCommand request, CancellationToken cancellationToken)
    {
        var album = await _albumRepository.GetByIdAsync(AlbumId.From(request.Id), cancellationToken);
        if (album == null)
            throw new InvalidOperationException($"Album with ID {request.Id} not found.");

        album.Rename(request.Name);
        album.UpdateDescription(request.Description);
        _albumRepository.Update(album);
        await _unitOfWork.SaveChangesAsync(cancellationToken);
        return Unit.Value;
    }
}

/// <summary>
/// Handler for DeleteAlbumCommand.
/// </summary>
public class DeleteAlbumCommandHandler : IRequestHandler<DeleteAlbumCommand, Unit>
{
    private readonly IAlbumRepository _albumRepository;
    private readonly IUnitOfWork _unitOfWork;

    public DeleteAlbumCommandHandler(
        IAlbumRepository albumRepository,
        IUnitOfWork unitOfWork)
    {
        _albumRepository = albumRepository;
        _unitOfWork = unitOfWork;
    }

    public async Task<Unit> Handle(DeleteAlbumCommand request, CancellationToken cancellationToken)
    {
        var album = await _albumRepository.GetByIdAsync(AlbumId.From(request.Id), cancellationToken);
        if (album == null)
            throw new InvalidOperationException($"Album with ID {request.Id} not found.");

        _albumRepository.Delete(album);
        await _unitOfWork.SaveChangesAsync(cancellationToken);
        return Unit.Value;
    }
}

/// <summary>
/// Handler for AddImageToAlbumCommand.
/// </summary>
public class AddImageToAlbumCommandHandler : IRequestHandler<AddImageToAlbumCommand, Unit>
{
    private readonly IAlbumRepository _albumRepository;
    private readonly IImageRepository _imageRepository;
    private readonly IUnitOfWork _unitOfWork;

    public AddImageToAlbumCommandHandler(
        IAlbumRepository albumRepository,
        IImageRepository imageRepository,
        IUnitOfWork unitOfWork)
    {
        _albumRepository = albumRepository;
        _imageRepository = imageRepository;
        _unitOfWork = unitOfWork;
    }

    public async Task<Unit> Handle(AddImageToAlbumCommand request, CancellationToken cancellationToken)
    {
        var album = await _albumRepository.GetByIdAsync(AlbumId.From(request.AlbumId), cancellationToken);
        if (album == null)
            throw new InvalidOperationException($"Album with ID {request.AlbumId} not found.");

        var image = await _imageRepository.GetByIdAsync(ImageId.From(request.ImageId), cancellationToken);
        if (image == null)
            throw new InvalidOperationException($"Image with ID {request.ImageId} not found.");

        album.AddImage(ImageId.From(request.ImageId));
        _albumRepository.Update(album);
        await _unitOfWork.SaveChangesAsync(cancellationToken);
        return Unit.Value;
    }
}

/// <summary>
/// Handler for RemoveImageFromAlbumCommand.
/// </summary>
public class RemoveImageFromAlbumCommandHandler : IRequestHandler<RemoveImageFromAlbumCommand, Unit>
{
    private readonly IAlbumRepository _albumRepository;
    private readonly IUnitOfWork _unitOfWork;

    public RemoveImageFromAlbumCommandHandler(
        IAlbumRepository albumRepository,
        IUnitOfWork unitOfWork)
    {
        _albumRepository = albumRepository;
        _unitOfWork = unitOfWork;
    }

    public async Task<Unit> Handle(RemoveImageFromAlbumCommand request, CancellationToken cancellationToken)
    {
        var album = await _albumRepository.GetByIdAsync(AlbumId.From(request.AlbumId), cancellationToken);
        if (album == null)
            throw new InvalidOperationException($"Album with ID {request.AlbumId} not found.");

        album.RemoveImage(ImageId.From(request.ImageId));
        _albumRepository.Update(album);
        await _unitOfWork.SaveChangesAsync(cancellationToken);
        return Unit.Value;
    }
}