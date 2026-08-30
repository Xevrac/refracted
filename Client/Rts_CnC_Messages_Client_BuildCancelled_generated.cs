using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_BuildCancelled
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.BuildCancelled); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.BuildCancelled)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize FactoryEntityId
            s.Write(value.FactoryEntityId);
            //  Serialize EntityType
            s.Write(value.EntityType);
            //  Serialize QueueIndex
            s.Write(value.QueueIndex);
            //  Serialize InsufficientResourceType
            s.Write(value.InsufficientResourceType);
            //  Serialize BothResourcesInsufficient
            s.Write(value.BothResourcesInsufficient);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.BuildCancelled)) as Rts.CnC.Messages.Client.BuildCancelled;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize FactoryEntityId
            s.Read(out value.FactoryEntityId);
            //  Deserialize EntityType
            s.Read(out value.EntityType);
            //  Deserialize QueueIndex
            s.Read(out value.QueueIndex);
            //  Deserialize InsufficientResourceType
            s.Read(out value.InsufficientResourceType);
            //  Deserialize BothResourcesInsufficient
            s.Read(out value.BothResourcesInsufficient);

            return value;
        }
        
    }
}
