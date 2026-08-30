using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntityOwnershipTransferred
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.EntityOwnershipTransferred); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.EntityOwnershipTransferred)obj;
            //  Serialize OldPlayerId
            s.Write(value.OldPlayerId);
            //  Serialize OldEntityId
            s.Write(value.OldEntityId);
            //  Serialize NewPlayerId
            s.Write(value.NewPlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.EntityOwnershipTransferred)) as Rts.CnC.Messages.Client.EntityOwnershipTransferred;
            //  Deserialize OldPlayerId
            s.Read(out value.OldPlayerId);
            //  Deserialize OldEntityId
            s.Read(out value.OldEntityId);
            //  Deserialize NewPlayerId
            s.Read(out value.NewPlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);

            return value;
        }
        
    }
}
