using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_PlacedBuildQueued
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.PlacedBuildQueued); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.PlacedBuildQueued)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize FactoryEntityId
            s.Write(value.FactoryEntityId);
            //  Serialize EntityToBeBuilt
            s.Write(value.EntityToBeBuilt);
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize Orientation
            s.Write(value.Orientation);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.PlacedBuildQueued)) as Rts.CnC.Messages.Client.PlacedBuildQueued;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize FactoryEntityId
            s.Read(out value.FactoryEntityId);
            //  Deserialize EntityToBeBuilt
            s.Read(out value.EntityToBeBuilt);
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize Orientation
            s.Read(out value.Orientation);

            return value;
        }
        
    }
}
