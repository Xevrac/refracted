using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_AbilityActivationFailure
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.AbilityActivationFailure); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.AbilityActivationFailure)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize AbilityId
            s.Write(value.AbilityId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.AbilityActivationFailure)) as Rts.CnC.Messages.Client.AbilityActivationFailure;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);

            return value;
        }
        
    }
}
