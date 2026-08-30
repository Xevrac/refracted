using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_AbilityReplaced
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.AbilityReplaced); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.AbilityReplaced)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize OldAbilityId
            s.Write(value.OldAbilityId);
            //  Serialize NewAbilityId
            s.Write(value.NewAbilityId);
            //  Serialize MillisecondsToReenable
            s.Write(value.MillisecondsToReenable);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.AbilityReplaced)) as Rts.CnC.Messages.Client.AbilityReplaced;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize OldAbilityId
            s.Read(out value.OldAbilityId);
            //  Deserialize NewAbilityId
            s.Read(out value.NewAbilityId);
            //  Deserialize MillisecondsToReenable
            s.Read(out value.MillisecondsToReenable);

            return value;
        }
        
    }
}
